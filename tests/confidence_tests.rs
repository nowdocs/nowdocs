//! Boundary tests for C05 binary-compatible answer states.
//!
//! These tests define the exact contract: the binary gate decision is exposed
//! as data (`AnswerDecision`) without changing any threshold or gate behavior.

use nowdocs::confidence::{decide_binary, AnswerState, DecisionReason, QueryEvidence};

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
