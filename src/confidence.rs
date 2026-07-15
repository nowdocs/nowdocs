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
