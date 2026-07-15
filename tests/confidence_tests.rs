//! Boundary tests for C05 binary-compatible answer states.
//!
//! These tests define the exact contract: the binary gate decision is exposed
//! as data (`AnswerDecision`) without changing any threshold or gate behavior.

use nowdocs::confidence::{
    decide_binary, decide_calibrated, AnswerState, DecisionReason, QueryEvidence,
    CALIBRATED_CONFIDENT_COSINE,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn empty_evidence() -> QueryEvidence {
    QueryEvidence {
        top_selected_cosine: None,
        top_selected_rrf: None,
        pre_mmr_top_cosine: None,
        pre_mmr_second_cosine: None,
        pre_mmr_cosine_margin: None,
        dense_rank: None,
        lexical_rank: None,
    }
}

// ---------------------------------------------------------------------------
// Boundary: just below cosine threshold → NoAnswer
// ---------------------------------------------------------------------------

#[test]
fn decide_below_cosine_threshold_is_no_answer() {
    let ev = QueryEvidence {
        top_selected_cosine: Some(0.8199),
        top_selected_rrf: Some(0.016),
        pre_mmr_top_cosine: Some(0.8199),
        pre_mmr_second_cosine: Some(0.75),
        pre_mmr_cosine_margin: Some(0.0699),
        dense_rank: Some(1),
        lexical_rank: None,
    };
    let d = decide_binary(&ev);
    assert_eq!(
        d.state,
        AnswerState::NoAnswer,
        "0.8199 cosine / 0.016 RRF / rank1 dense only must be NoAnswer"
    );
    assert_eq!(d.reason, DecisionReason::CurrentGateReject);
}

// ---------------------------------------------------------------------------
// Boundary: exactly at cosine threshold → Confident
// ---------------------------------------------------------------------------

#[test]
fn decide_at_cosine_threshold_is_confident() {
    let ev = QueryEvidence {
        top_selected_cosine: Some(0.82),
        top_selected_rrf: Some(0.016),
        pre_mmr_top_cosine: Some(0.82),
        pre_mmr_second_cosine: Some(0.75),
        pre_mmr_cosine_margin: Some(0.07),
        dense_rank: Some(1),
        lexical_rank: None,
    };
    let d = decide_binary(&ev);
    assert_eq!(
        d.state,
        AnswerState::Confident,
        "0.82 cosine must be Confident (exact threshold)"
    );
    assert_eq!(d.reason, DecisionReason::CurrentCosinePass);
}

// ---------------------------------------------------------------------------
// Dual-channel rank-1 bypass: low cosine, high RRF → Confident
// ---------------------------------------------------------------------------

#[test]
fn decide_dual_rank1_bypass_is_confident() {
    let ev = QueryEvidence {
        top_selected_cosine: Some(0.75),
        top_selected_rrf: Some(0.0332),
        pre_mmr_top_cosine: Some(0.75),
        pre_mmr_second_cosine: Some(0.70),
        pre_mmr_cosine_margin: Some(0.05),
        dense_rank: Some(1),
        lexical_rank: Some(1),
    };
    let d = decide_binary(&ev);
    assert_eq!(
        d.state,
        AnswerState::Confident,
        "0.75 cosine / 0.0332 RRF / rank1 in both channels must be Confident via dual-rank-1 bypass"
    );
    assert_eq!(d.reason, DecisionReason::CurrentDualRankPass);
}

// ---------------------------------------------------------------------------
// No candidates → NoAnswer with NoCandidates reason
// ---------------------------------------------------------------------------

#[test]
fn decide_no_candidates_is_no_answer() {
    let ev = empty_evidence();
    let d = decide_binary(&ev);
    assert_eq!(d.state, AnswerState::NoAnswer);
    assert_eq!(d.reason, DecisionReason::NoCandidates);
}

// ---------------------------------------------------------------------------
// Gate reject: below cosine AND below dual-rank-1 RRF
// ---------------------------------------------------------------------------

#[test]
fn decide_below_both_thresholds_is_gate_reject() {
    let ev = QueryEvidence {
        top_selected_cosine: Some(0.80),
        top_selected_rrf: Some(0.016),
        pre_mmr_top_cosine: Some(0.80),
        pre_mmr_second_cosine: Some(0.75),
        pre_mmr_cosine_margin: Some(0.05),
        dense_rank: Some(1),
        lexical_rank: None,
    };
    let d = decide_binary(&ev);
    assert_eq!(d.state, AnswerState::NoAnswer);
    assert_eq!(d.reason, DecisionReason::CurrentGateReject);
}

#[test]
fn decide_preserves_legacy_gate_behavior_for_nan_cosine() {
    let ev = QueryEvidence {
        top_selected_cosine: Some(f32::NAN),
        top_selected_rrf: Some(0.016),
        pre_mmr_top_cosine: None,
        pre_mmr_second_cosine: None,
        pre_mmr_cosine_margin: None,
        dense_rank: None,
        lexical_rank: None,
    };

    // `apply_answer_gate` uses `<` for its rejection condition. IEEE NaN is
    // not less than the floor, so the legacy gate accepts this defensive path.
    assert_eq!(decide_binary(&ev).state, AnswerState::Confident);
}

// ---------------------------------------------------------------------------
// Serde: exact key contract
// ---------------------------------------------------------------------------

#[test]
fn answer_state_serde_keys_are_exact() {
    let cases = [
        (AnswerState::Confident, "\"confident\""),
        (AnswerState::Borderline, "\"borderline\""),
        (AnswerState::NoAnswer, "\"no_answer\""),
    ];
    for (state, expected) in &cases {
        let json = serde_json::to_string(state).expect("serialize");
        assert_eq!(&json, expected, "serde key mismatch for {state:?}");
        let back: AnswerState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, *state);
    }
}

// ---------------------------------------------------------------------------
// C07b: calibrated three-state decision tests
// ---------------------------------------------------------------------------
//
// The calibrated decision derives the existing binary gate first, then lowers
// the certainty of accepted results below the calibrated confidence floor.
// Only NoAnswer returns an empty chunk list; Borderline retains the selected
// hits.

/// The calibrated confidence floor constant must be exactly 0.845.
#[test]
fn calibrated_confident_cosine_constant_is_exact() {
    assert_eq!(CALIBRATED_CONFIDENT_COSINE, 0.845);
}

/// No candidates -> NoAnswer / NoCandidates.
#[test]
fn calibrated_no_candidates_is_no_answer() {
    let d = decide_calibrated(&empty_evidence());
    assert_eq!(d.state, AnswerState::NoAnswer);
    assert_eq!(d.reason, DecisionReason::NoCandidates);
}

/// Binary rejection at 0.8199 / low RRF -> NoAnswer / CalibratedNoAnswer.
#[test]
fn calibrated_binary_reject_is_calibrated_no_answer() {
    let ev = QueryEvidence {
        top_selected_cosine: Some(0.8199),
        top_selected_rrf: Some(0.016),
        pre_mmr_top_cosine: Some(0.8199),
        pre_mmr_second_cosine: Some(0.75),
        pre_mmr_cosine_margin: Some(0.0699),
        dense_rank: Some(1),
        lexical_rank: None,
    };
    let d = decide_calibrated(&ev);
    assert_eq!(
        d.state,
        AnswerState::NoAnswer,
        "binary rejection must remain NoAnswer"
    );
    assert_eq!(
        d.reason,
        DecisionReason::CalibratedNoAnswer,
        "nonempty-fused NoAnswer must be CalibratedNoAnswer"
    );
}

/// 0.82 accepted by the old gate but below 0.845 -> Borderline /
/// CalibratedBorderline.
#[test]
fn calibrated_at_binary_threshold_below_floor_is_borderline() {
    let ev = QueryEvidence {
        top_selected_cosine: Some(0.82),
        top_selected_rrf: Some(0.016),
        pre_mmr_top_cosine: Some(0.82),
        pre_mmr_second_cosine: Some(0.75),
        pre_mmr_cosine_margin: Some(0.07),
        dense_rank: Some(1),
        lexical_rank: None,
    };
    let d = decide_calibrated(&ev);
    assert_eq!(
        d.state,
        AnswerState::Borderline,
        "0.82 accepted by binary gate but below 0.845 must be Borderline"
    );
    assert_eq!(d.reason, DecisionReason::CalibratedBorderline);
}

/// 0.8449 (just below the calibrated floor) -> Borderline.
#[test]
fn calibrated_just_below_floor_is_borderline() {
    let ev = QueryEvidence {
        top_selected_cosine: Some(0.8449),
        top_selected_rrf: Some(0.016),
        pre_mmr_top_cosine: Some(0.8449),
        pre_mmr_second_cosine: Some(0.75),
        pre_mmr_cosine_margin: Some(0.0949),
        dense_rank: Some(1),
        lexical_rank: None,
    };
    let d = decide_calibrated(&ev);
    assert_eq!(d.state, AnswerState::Borderline);
    assert_eq!(d.reason, DecisionReason::CalibratedBorderline);
}

/// Exactly 0.845 -> Confident / CalibratedConfident (inclusive floor).
#[test]
fn calibrated_at_floor_is_confident() {
    let ev = QueryEvidence {
        top_selected_cosine: Some(0.845),
        top_selected_rrf: Some(0.016),
        pre_mmr_top_cosine: Some(0.845),
        pre_mmr_second_cosine: Some(0.75),
        pre_mmr_cosine_margin: Some(0.095),
        dense_rank: Some(1),
        lexical_rank: None,
    };
    let d = decide_calibrated(&ev);
    assert_eq!(
        d.state,
        AnswerState::Confident,
        "exactly 0.845 (inclusive) must be Confident"
    );
    assert_eq!(d.reason, DecisionReason::CalibratedConfident);
}

/// A high cosine above the floor -> Confident / CalibratedConfident.
#[test]
fn calibrated_above_floor_is_confident() {
    let ev = QueryEvidence {
        top_selected_cosine: Some(0.90),
        top_selected_rrf: Some(0.016),
        pre_mmr_top_cosine: Some(0.90),
        pre_mmr_second_cosine: Some(0.75),
        pre_mmr_cosine_margin: Some(0.15),
        dense_rank: Some(1),
        lexical_rank: None,
    };
    let d = decide_calibrated(&ev);
    assert_eq!(d.state, AnswerState::Confident);
    assert_eq!(d.reason, DecisionReason::CalibratedConfident);
}

/// Dual-rank-1 legacy bypass below 0.845 -> Borderline.
#[test]
fn calibrated_dual_rank1_bypass_below_floor_is_borderline() {
    let ev = QueryEvidence {
        top_selected_cosine: Some(0.75),
        top_selected_rrf: Some(0.0332),
        pre_mmr_top_cosine: Some(0.75),
        pre_mmr_second_cosine: Some(0.70),
        pre_mmr_cosine_margin: Some(0.05),
        dense_rank: Some(1),
        lexical_rank: Some(1),
    };
    let d = decide_calibrated(&ev);
    assert_eq!(
        d.state,
        AnswerState::Borderline,
        "dual-rank-1 bypass below 0.845 must be Borderline, not Confident"
    );
    assert_eq!(d.reason, DecisionReason::CalibratedBorderline);
}

/// A NaN cosine accepted by `decide_binary` -> Borderline in the calibrated
/// decision (non-finite cosine is never Confident).
#[test]
fn calibrated_nan_cosine_accepted_by_binary_is_borderline() {
    let ev = QueryEvidence {
        top_selected_cosine: Some(f32::NAN),
        top_selected_rrf: Some(0.016),
        pre_mmr_top_cosine: None,
        pre_mmr_second_cosine: None,
        pre_mmr_cosine_margin: None,
        dense_rank: None,
        lexical_rank: None,
    };
    // The legacy binary gate accepts NaN (its `<` rejection is not met).
    assert_eq!(decide_binary(&ev).state, AnswerState::Confident);
    // The calibrated decision must lower NaN to Borderline, never Confident.
    let d = decide_calibrated(&ev);
    assert_eq!(
        d.state,
        AnswerState::Borderline,
        "NaN cosine accepted by binary must be Borderline, never Confident"
    );
    assert_eq!(d.reason, DecisionReason::CalibratedBorderline);
}

/// The calibrated decision must not alter the binary gate's constants: it
/// derives the binary decision first and only relabels accepted results.
#[test]
fn calibrated_preserves_binary_reject_constants() {
    // 0.80 / 0.016: below both binary thresholds -> binary reject.
    let ev = QueryEvidence {
        top_selected_cosine: Some(0.80),
        top_selected_rrf: Some(0.016),
        pre_mmr_top_cosine: Some(0.80),
        pre_mmr_second_cosine: Some(0.75),
        pre_mmr_cosine_margin: Some(0.05),
        dense_rank: Some(1),
        lexical_rank: None,
    };
    let b = decide_binary(&ev);
    let c = decide_calibrated(&ev);
    assert_eq!(b.state, AnswerState::NoAnswer);
    assert_eq!(c.state, AnswerState::NoAnswer);
    assert_eq!(c.reason, DecisionReason::CalibratedNoAnswer);
}
