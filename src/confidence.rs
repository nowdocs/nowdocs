//! Confidence labels for retrieval answers.
//!
//! `AnswerState` is the serialized verdict later runtime work attaches to
//! every search answer: whether the pipeline is confident, borderline, or has
//! no answer at all. The serde representation (`confident` / `borderline` /
//! `no_answer`) is a stable contract — do not rename variants without a
//! migration.

/// Confidence state of a retrieval answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswerState {
    Confident,
    Borderline,
    NoAnswer,
}

/// Evidence collected from the retrieval pipeline for a single query.
/// Populated after MMR selection, before neighbor expansion.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryEvidence {
    pub top_selected_cosine: Option<f32>,
    pub top_selected_rrf: Option<f32>,
    pub pre_mmr_top_cosine: Option<f32>,
    pub pre_mmr_second_cosine: Option<f32>,
    pub pre_mmr_cosine_margin: Option<f32>,
    pub dense_rank: Option<u32>,
    pub lexical_rank: Option<u32>,
}

/// Reason for the binary answer-state decision. Maps 1:1 to the current
/// gate's acceptance/rejection paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionReason {
    NoCandidates,
    CurrentCosinePass,
    CurrentDualRankPass,
    CurrentGateReject,
    CalibratedConfident,
    CalibratedBorderline,
    CalibratedNoAnswer,
}

/// Binary answer-state decision: the state plus the reason for that state.
pub struct AnswerDecision {
    pub state: AnswerState,
    pub reason: DecisionReason,
}

/// Binary gate decision: exposes the current cosine/RRF gate as data.
///
/// For every existing query outcome, its state is binary-equivalent to the
/// pre-C05 gate: `Confident` when the current gate accepted, `NoAnswer` when
/// it rejected. `Borderline` is never returned at runtime.
pub fn decide_binary(evidence: &QueryEvidence) -> AnswerDecision {
    if evidence.top_selected_cosine.is_none() {
        return AnswerDecision {
            state: AnswerState::NoAnswer,
            reason: DecisionReason::NoCandidates,
        };
    }

    let top_cosine = evidence.top_selected_cosine.unwrap();
    let top_rrf = evidence.top_selected_rrf.unwrap_or(0.0);

    if top_cosine < crate::retrieve::MIN_ANSWER_COSINE && top_rrf < crate::retrieve::DUAL_RANK1_RRF
    {
        return AnswerDecision {
            state: AnswerState::NoAnswer,
            reason: DecisionReason::CurrentGateReject,
        };
    }

    if top_cosine >= crate::retrieve::MIN_ANSWER_COSINE {
        return AnswerDecision {
            state: AnswerState::Confident,
            reason: DecisionReason::CurrentCosinePass,
        };
    }

    // This branch also retains the legacy IEEE-754 NaN behavior: the gate's
    // rejection predicate uses `<`, so NaN does not meet it and is accepted.
    // No dedicated reason exists for that defensive path; it shares the
    // secondary-pass classification to preserve binary behavior exactly.
    AnswerDecision {
        state: AnswerState::Confident,
        reason: DecisionReason::CurrentDualRankPass,
    }
}

// ---------------------------------------------------------------------------
// C07b: calibrated three-state decision
// ---------------------------------------------------------------------------

/// Calibrated confidence floor: a selected hit whose raw cosine to the query
/// reaches this value (inclusive) is `Confident` under the calibrated policy.
/// Below it, a binary-gate-accepted hit is `Borderline`.
///
/// Selected on the development split only (C07a evidence) and must not be
/// tuned against the test split. Ownership lives beside the calibrated decision
/// so retrieval and tests reference this constant rather than copying the
/// literal.
pub const CALIBRATED_CONFIDENT_COSINE: f32 = 0.845;

/// Calibrated three-state decision (C07b): derives the existing binary
/// decision first, then lowers the certainty of accepted results below the
/// calibrated confidence floor.
///
/// The policy is:
///
/// ```text
/// no candidates                                        -> NoAnswer / NoCandidates
/// existing binary gate rejects                         -> NoAnswer / CalibratedNoAnswer
/// existing binary gate accepts; finite cosine >= floor -> Confident / CalibratedConfident
/// all other existing-binary-gate accepts               -> Borderline / CalibratedBorderline
/// ```
///
/// `CALIBRATED_CONFIDENT_COSINE` is inclusive. A non-finite selected cosine
/// is never `Confident`: if the legacy binary gate accepts that defensive
/// IEEE-754 path (NaN is not `<` the floor, so it is accepted), the calibrated
/// decision labels it `Borderline`. Only `NoAnswer` returns an empty chunk
/// list; `Borderline` retains the selected chunks and neighbor context.
///
/// This function does not duplicate or reimplement the binary gate constants:
/// it delegates to [`decide_binary`] and only relabels the accepted states.
pub fn decide_calibrated(evidence: &QueryEvidence) -> AnswerDecision {
    let binary = decide_binary(evidence);
    match binary.state {
        AnswerState::NoAnswer => {
            // Preserve the no-candidates distinction; a gate reject becomes
            // CalibratedNoAnswer (nonempty-fused) rather than CurrentGateReject.
            let reason = if binary.reason == DecisionReason::NoCandidates {
                DecisionReason::NoCandidates
            } else {
                DecisionReason::CalibratedNoAnswer
            };
            AnswerDecision {
                state: AnswerState::NoAnswer,
                reason,
            }
        }
        AnswerState::Confident => {
            // The binary gate accepted. Only a finite cosine at or above the
            // floor is Confident; everything else (including NaN and the
            // dual-rank-1 bypass below the floor) is Borderline.
            let top_cosine = evidence.top_selected_cosine.unwrap_or(f32::NAN);
            if top_cosine.is_finite() && top_cosine >= CALIBRATED_CONFIDENT_COSINE {
                AnswerDecision {
                    state: AnswerState::Confident,
                    reason: DecisionReason::CalibratedConfident,
                }
            } else {
                AnswerDecision {
                    state: AnswerState::Borderline,
                    reason: DecisionReason::CalibratedBorderline,
                }
            }
        }
        // decide_binary never returns Borderline, but cover the case for
        // completeness: a future caller cannot reach here.
        AnswerState::Borderline => AnswerDecision {
            state: AnswerState::Borderline,
            reason: DecisionReason::CalibratedBorderline,
        },
    }
}
