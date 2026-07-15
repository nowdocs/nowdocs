//! Contract tests for the versioned JSON retrieval evaluator report (C03).
//!
//! These tests pin the machine-readable v1 report shape: stable top-level
//! field names, the exact stage-key set, snake-case decision reasons, and the
//! guarantee that no query text or chunk text ever appears in a serialized
//! query report. They are pure serialization/CLI-parse tests — no embedder,
//! no store, no installed corpus required.

use std::collections::{BTreeMap, BTreeSet};

use nowdocs::confidence::AnswerState;
use nowdocs::eval::{
    answer_state_to_report, rate_estimate, validate_evidence_output_path, AnswerStateMetrics,
    CorpusIdentity, DecisionEvidenceRow, EvalReportV1, EvalSplit, MetricBucket, QueryClass,
    QueryForm, QueryReport, ReportDecisionReason, RetrievalEvalArgs, RetrievalParameters,
    StageMetrics, StageMetricsSet, StageRanks, CALIBRATED_POLICY_ID,
};

fn sample_stage(k: usize) -> StageMetrics {
    StageMetrics {
        k,
        recall: 0.5,
        mrr: 0.5,
        ndcg: 0.5,
        precision: 0.5,
    }
}

fn sample_stages() -> StageMetricsSet {
    StageMetricsSet {
        dense_at_40: sample_stage(40),
        fts_at_40: sample_stage(40),
        fused_at_40: sample_stage(40),
        mmr_at_5: sample_stage(5),
        output_at_5: sample_stage(5),
    }
}

fn sample_answer_states() -> AnswerStateMetrics {
    AnswerStateMetrics {
        false_reject: rate_estimate(1, 4),
        false_accept: rate_estimate(0, 2),
        positive_borderline: rate_estimate(0, 4),
        negative_borderline: rate_estimate(0, 2),
        decisive_coverage: rate_estimate(6, 6),
    }
}

fn sample_report() -> EvalReportV1 {
    let bucket = MetricBucket {
        stages: sample_stages(),
        answer_states: sample_answer_states(),
    };
    let mut by_docset = BTreeMap::new();
    by_docset.insert("nextjs".to_string(), bucket.clone());
    let mut by_query_form = BTreeMap::new();
    by_query_form.insert("natural_language".to_string(), bucket.clone());
    let mut by_query_class = BTreeMap::new();
    by_query_class.insert("positive".to_string(), bucket);

    let query = QueryReport {
        id: "nextjs-dev-001".to_string(),
        docset: "nextjs".to_string(),
        split: EvalSplit::Development,
        form: QueryForm::NaturalLanguage,
        class: QueryClass::Positive,
        matched_target_indexes: vec![0],
        matched_target_grades: vec![2],
        stage_ranks: StageRanks {
            dense_at_40: Some(3),
            fts_at_40: None,
            fused_at_40: Some(2),
            mmr_at_5: Some(1),
            output_at_5: Some(1),
        },
        answer_state: AnswerState::Confident,
        decision_reason: ReportDecisionReason::CurrentGatePass,
    };

    EvalReportV1 {
        schema_version: 1,
        corpora: vec![CorpusIdentity {
            docset: "nextjs".to_string(),
            code_commit: "2539729d317def57fee4e30cb6cea8172f1d02aa".to_string(),
            command: "retrieval_eval --split development".to_string(),
            parameters: RetrievalParameters {
                answer_policy: nowdocs::eval::BINARY_GATE_POLICY_ID.to_string(),
                max_tokens: 4000,
                top_k: 5,
                candidate_pool_k: 40,
                rrf_fusion_k: 60.0,
                min_answer_cosine: nowdocs::retrieve::MIN_ANSWER_COSINE,
                dual_rank1_rrf: nowdocs::retrieve::DUAL_RANK1_RRF,
                split: EvalSplit::Development,
                benchmark_runs: 1,
            },
            os: "macos".to_string(),
            arch: "aarch64".to_string(),
            manifest_sha256: "ab".repeat(32),
            document_version: "2026-07-01".to_string(),
            chunk_count: 7480,
            model_revision: "44e7d1d7a7a56f64baaf5e4b0d3f6a2b1a2b3c4d".to_string(),
            model_hash: "cd".repeat(32),
        }],
        stages: sample_stages(),
        answer_states: sample_answer_states(),
        by_docset,
        by_query_form,
        by_query_class,
        queries: vec![query],
    }
}

/// Recursively assert that no object below `value` carries a field named
/// `forbidden`. Used to prove query reports can never leak query text or
/// chunk text under any future field addition.
fn assert_no_field_named(value: &serde_json::Value, forbidden: &str) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                assert!(
                    key != forbidden,
                    "query report must not serialize a field named {forbidden:?}, found at key {key:?}"
                );
                assert_no_field_named(child, forbidden);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                assert_no_field_named(item, forbidden);
            }
        }
        _ => {}
    }
}

/// Fixture report test (C03): the v1 report serializes the machine-readable
/// contract — schema_version == 1, array corpora/queries, object
/// stages.fused_at_40 and answer_states.false_reject, and no query-report
/// field named `query` or `text`.
#[test]
fn report_v1_serializes_machine_readable_contract() {
    let value = serde_json::to_value(sample_report()).expect("serialize EvalReportV1");

    assert_eq!(
        value["schema_version"],
        serde_json::json!(1),
        "schema_version must be the integer 1"
    );

    // Required top-level fields with stable names.
    let expected_top: BTreeSet<&str> = [
        "schema_version",
        "corpora",
        "stages",
        "answer_states",
        "by_docset",
        "by_query_form",
        "by_query_class",
        "queries",
    ]
    .into_iter()
    .collect();
    let actual_top: BTreeSet<&str> = value
        .as_object()
        .expect("report is an object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(actual_top, expected_top, "top-level field set must match");

    assert!(value["corpora"].is_array(), "corpora must be an array");
    assert!(value["queries"].is_array(), "queries must be an array");
    assert!(
        value["stages"]["fused_at_40"].is_object(),
        "stages.fused_at_40 must be an object"
    );
    assert!(
        value["answer_states"]["false_reject"].is_object(),
        "answer_states.false_reject must be an object"
    );
    assert_eq!(
        value["corpora"][0]["command"],
        serde_json::json!("retrieval_eval --split development"),
        "corpus identity must record the evaluator command"
    );

    // The same stage/state structure is emitted per docset, form, and class.
    for group in ["by_docset", "by_query_form", "by_query_class"] {
        let buckets = value[group].as_object().expect("group is an object");
        assert!(!buckets.is_empty(), "sample {group} must be non-empty");
        for (key, bucket) in buckets {
            assert!(
                bucket["stages"]["fused_at_40"].is_object(),
                "{group}.{key}.stages.fused_at_40 must be an object"
            );
            assert!(
                bucket["answer_states"]["false_reject"].is_object(),
                "{group}.{key}.answer_states.false_reject must be an object"
            );
        }
    }

    // No serialized query-report field is named `query` or `text`, at any depth.
    for query_report in value["queries"].as_array().expect("queries is an array") {
        assert_no_field_named(query_report, "query");
        assert_no_field_named(query_report, "text");
    }
}

/// The `stages` object must contain exactly the five pinned stage keys — no
/// more, no fewer — at the top level and inside every bucket.
#[test]
fn report_v1_stage_key_set_is_exact() {
    let value = serde_json::to_value(sample_report()).expect("serialize EvalReportV1");
    let expected: BTreeSet<&str> = [
        "dense_at_40",
        "fts_at_40",
        "fused_at_40",
        "mmr_at_5",
        "output_at_5",
    ]
    .into_iter()
    .collect();

    let assert_keys = |stages: &serde_json::Value, where_: &str| {
        let actual: BTreeSet<&str> = stages
            .as_object()
            .expect("stages is an object")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(actual, expected, "stage key set must be exact at {where_}");
    };

    assert_keys(&value["stages"], "top level");
    for group in ["by_docset", "by_query_form", "by_query_class"] {
        for (key, bucket) in value[group].as_object().expect("group is an object") {
            assert_keys(&bucket["stages"], &format!("{group}.{key}"));
        }
    }

    // Each stage reports k, recall, mrr, ndcg, and precision.
    let stage = &value["stages"]["fused_at_40"];
    for field in ["k", "recall", "mrr", "ndcg", "precision"] {
        assert!(
            stage.get(field).is_some_and(|v| v.is_number()),
            "stage must report numeric {field}"
        );
    }
}

/// Every ReportDecisionReason variant serializes to its exact snake-case
/// report name, including the calibrated variants reserved for later policies.
#[test]
fn report_decision_reason_serializes_snake_case() {
    let cases = [
        (ReportDecisionReason::NoCandidates, "no_candidates"),
        (ReportDecisionReason::CurrentGatePass, "current_gate_pass"),
        (
            ReportDecisionReason::CurrentGateReject,
            "current_gate_reject",
        ),
        (
            ReportDecisionReason::CalibratedConfident,
            "calibrated_confident",
        ),
        (
            ReportDecisionReason::CalibratedBorderline,
            "calibrated_borderline",
        ),
        (
            ReportDecisionReason::CalibratedNoAnswer,
            "calibrated_no_answer",
        ),
    ];
    for (variant, expected) in cases {
        let value = serde_json::to_value(variant).expect("serialize reason");
        assert_eq!(
            value,
            serde_json::json!(expected),
            "variant must serialize as {expected}"
        );
    }
}

/// CLI contract: `--code-commit` is validated as a 40-character hexadecimal
/// SHA at argument-parse time, so an invalid commit is rejected before the
/// evaluator loads fixtures, a model, or a store. Also pins `--split`
/// validation and the `--benchmark-runs` default/lower bound.
#[test]
fn cli_rejects_invalid_code_commit_without_model_initialization() {
    let base: [&str; 7] = [
        "retrieval_eval",
        "--fixtures-dir",
        "tests/fixtures/eval",
        "--split",
        "development",
        "--output",
        "/tmp/retrieval-eval-report.json",
    ];
    let mut valid = base.to_vec();
    valid.extend(["--code-commit", "2539729d317def57fee4e30cb6cea8172f1d02aa"]);
    let parsed = RetrievalEvalArgs::try_parse_args(&valid).expect("valid SHA parses");
    assert_eq!(parsed.benchmark_runs, 1, "benchmark_runs defaults to 1");

    // Too short, too long, non-hex, and empty commits are all rejected at
    // parse time — the evaluator binary exits before any model/store work.
    for bad in [
        "",
        "2539729",
        "2539729d317def57fee4e30cb6cea8172f1d02aa00",
        "g539729d317def57fee4e30cb6cea8172f1d02aa",
    ] {
        let mut args = base.to_vec();
        args.extend(["--code-commit", bad]);
        assert!(
            RetrievalEvalArgs::try_parse_args(&args).is_err(),
            "invalid --code-commit {bad:?} must be rejected at parse time"
        );
    }

    // Invalid split values and non-positive benchmark counts are rejected too.
    let mut bad_split: Vec<&str> = base.to_vec();
    bad_split[4] = "staging";
    bad_split.extend(["--code-commit", "2539729d317def57fee4e30cb6cea8172f1d02aa"]);
    assert!(
        RetrievalEvalArgs::try_parse_args(&bad_split).is_err(),
        "--split staging must be rejected"
    );

    let mut zero_runs = base.to_vec();
    zero_runs.extend([
        "--code-commit",
        "2539729d317def57fee4e30cb6cea8172f1d02aa",
        "--benchmark-runs",
        "0",
    ]);
    assert!(
        RetrievalEvalArgs::try_parse_args(&zero_runs).is_err(),
        "--benchmark-runs 0 must be rejected"
    );

    let mut excessive_runs = base.to_vec();
    excessive_runs.extend([
        "--code-commit",
        "2539729d317def57fee4e30cb6cea8172f1d02aa",
        "--benchmark-runs",
        "1001",
    ]);
    assert!(
        RetrievalEvalArgs::try_parse_args(&excessive_runs).is_err(),
        "--benchmark-runs above the supported maximum must be rejected"
    );
}

// ---------------------------------------------------------------------------
// C06/C07b: answer_state_to_report mapping tests
// ---------------------------------------------------------------------------

/// Regression: the report mapping must follow `SearchResult.answer_state`,
/// not the trace `gate_passed` bit. This test supplies a `NoAnswer` state
/// with an empty fused trace and expects `NoCandidates`.
#[test]
fn answer_state_no_answer_empty_fused_is_no_candidates() {
    let (state, reason) = answer_state_to_report(AnswerState::NoAnswer, true);
    assert_eq!(state, AnswerState::NoAnswer);
    assert_eq!(reason, ReportDecisionReason::NoCandidates);
}

/// Regression: `NoAnswer` with a non-empty fused trace maps to
/// `CalibratedNoAnswer` under the C07b calibrated policy (gate saw candidates
/// but rejected them).
#[test]
fn answer_state_no_answer_non_empty_fused_is_calibrated_no_answer() {
    let (state, reason) = answer_state_to_report(AnswerState::NoAnswer, false);
    assert_eq!(state, AnswerState::NoAnswer);
    assert_eq!(reason, ReportDecisionReason::CalibratedNoAnswer);
}

/// Regression: `Confident` always maps to `CalibratedConfident` regardless of
/// the trace's fused-pool contents under the C07b calibrated policy.
#[test]
fn answer_state_confident_is_calibrated_confident() {
    let (state, reason) = answer_state_to_report(AnswerState::Confident, true);
    assert_eq!(state, AnswerState::Confident);
    assert_eq!(reason, ReportDecisionReason::CalibratedConfident);

    let (state, reason) = answer_state_to_report(AnswerState::Confident, false);
    assert_eq!(state, AnswerState::Confident);
    assert_eq!(reason, ReportDecisionReason::CalibratedConfident);
}

/// Regression: `Borderline` maps to `CalibratedBorderline` under the C07b
/// calibrated policy.
#[test]
fn answer_state_borderline_is_reserved_calibrated() {
    let (state, reason) = answer_state_to_report(AnswerState::Borderline, false);
    assert_eq!(state, AnswerState::Borderline);
    assert_eq!(reason, ReportDecisionReason::CalibratedBorderline);
}

/// Key regression: when `SearchResult.answer_state` contradicts the trace
/// `gate_passed` bit, the report must follow the result state. Here the
/// result says `NoAnswer` but the trace gate would say passed - the report
/// must reflect `NoAnswer`.
#[test]
fn answer_state_follows_result_not_trace_gate() {
    // Simulate: trace.gate_passed = true (fused non-empty, gate passed),
    // but result.answer_state = NoAnswer. The report must follow the result.
    let (state, reason) = answer_state_to_report(AnswerState::NoAnswer, false);
    assert_eq!(
        state,
        AnswerState::NoAnswer,
        "report must use result.answer_state, not trace.gate_passed"
    );
    assert_eq!(
        reason,
        ReportDecisionReason::CalibratedNoAnswer,
        "non-empty fused + NoAnswer must be CalibratedNoAnswer under the calibrated policy"
    );
}

// ---------------------------------------------------------------------------
// C07a: decision-evidence sidecar tests
// ---------------------------------------------------------------------------

/// `--evidence-output` accepts an absolute path at parse time.
#[test]
fn cli_accepts_absolute_evidence_output() {
    let args = RetrievalEvalArgs::try_parse_args(&[
        "retrieval_eval",
        "--fixtures-dir",
        "tests/fixtures/eval",
        "--split",
        "development",
        "--output",
        "/tmp/report.json",
        "--code-commit",
        "2539729d317def57fee4e30cb6cea8172f1d02aa",
        "--evidence-output",
        "/tmp/evidence.json",
    ])
    .expect("--evidence-output with absolute path must parse");
    assert_eq!(
        args.evidence_output,
        Some(std::path::PathBuf::from("/tmp/evidence.json"))
    );
}

/// `--evidence-output` rejects a relative path at parse time, before any
/// model or store work.
#[test]
fn cli_rejects_relative_evidence_output() {
    let result = RetrievalEvalArgs::try_parse_args(&[
        "retrieval_eval",
        "--fixtures-dir",
        "tests/fixtures/eval",
        "--split",
        "development",
        "--output",
        "/tmp/report.json",
        "--code-commit",
        "2539729d317def57fee4e30cb6cea8172f1d02aa",
        "--evidence-output",
        "relative/path.json",
    ]);
    assert!(
        result.is_err(),
        "--evidence-output with relative path must be rejected"
    );
}

#[test]
fn evidence_output_must_differ_from_report_output() {
    let path = std::path::Path::new("/tmp/eval.json");
    assert!(
        validate_evidence_output_path(path, Some(path)).is_err(),
        "sidecar must not overwrite the primary report"
    );
    assert!(
        validate_evidence_output_path(path, Some(std::path::Path::new("/tmp/evidence.json")))
            .is_ok()
    );
}

/// `DecisionEvidenceRow` serializes with exactly the required keys.
#[test]
fn evidence_row_keys_are_exact() {
    let row = DecisionEvidenceRow {
        id: "q1".to_string(),
        docset: "nextjs".to_string(),
        split: EvalSplit::Development,
        form: QueryForm::NaturalLanguage,
        class: QueryClass::Positive,
        answer_state: AnswerState::Confident,
        decision_reason: ReportDecisionReason::CurrentGatePass,
        top_selected_cosine: Some(0.85),
        top_selected_rrf: Some(0.03),
        pre_mmr_top_cosine: Some(0.90),
        pre_mmr_second_cosine: Some(0.80),
        pre_mmr_cosine_margin: Some(0.10),
        dense_rank: Some(2),
        lexical_rank: Some(5),
    };
    let value = serde_json::to_value(&row).expect("serialize DecisionEvidenceRow");
    let expected: BTreeSet<&str> = [
        "id",
        "docset",
        "split",
        "form",
        "class",
        "answer_state",
        "decision_reason",
        "top_selected_cosine",
        "top_selected_rrf",
        "pre_mmr_top_cosine",
        "pre_mmr_second_cosine",
        "pre_mmr_cosine_margin",
        "dense_rank",
        "lexical_rank",
    ]
    .into_iter()
    .collect();
    let actual: BTreeSet<&str> = value
        .as_object()
        .expect("row is an object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(actual, expected, "evidence row key set must be exact");
}

/// Null values serialize as JSON `null`, not omitted or zero.
#[test]
fn evidence_row_nulls_serialize_correctly() {
    let row = DecisionEvidenceRow {
        id: "q1".to_string(),
        docset: "nextjs".to_string(),
        split: EvalSplit::Development,
        form: QueryForm::NaturalLanguage,
        class: QueryClass::Positive,
        answer_state: AnswerState::NoAnswer,
        decision_reason: ReportDecisionReason::NoCandidates,
        top_selected_cosine: None,
        top_selected_rrf: None,
        pre_mmr_top_cosine: None,
        pre_mmr_second_cosine: None,
        pre_mmr_cosine_margin: None,
        dense_rank: None,
        lexical_rank: None,
    };
    let value = serde_json::to_value(&row).expect("serialize DecisionEvidenceRow");
    for field in [
        "top_selected_cosine",
        "top_selected_rrf",
        "pre_mmr_top_cosine",
        "pre_mmr_second_cosine",
        "pre_mmr_cosine_margin",
        "dense_rank",
        "lexical_rank",
    ] {
        assert!(
            value[field].is_null(),
            "{field} must serialize as null when absent"
        );
    }
}

/// Evidence rows must never contain query text or chunk text.
#[test]
fn evidence_row_contains_no_query_or_chunk_text() {
    let row = DecisionEvidenceRow {
        id: "q1".to_string(),
        docset: "nextjs".to_string(),
        split: EvalSplit::Development,
        form: QueryForm::NaturalLanguage,
        class: QueryClass::Positive,
        answer_state: AnswerState::Confident,
        decision_reason: ReportDecisionReason::CurrentGatePass,
        top_selected_cosine: Some(0.85),
        top_selected_rrf: Some(0.03),
        pre_mmr_top_cosine: Some(0.90),
        pre_mmr_second_cosine: Some(0.80),
        pre_mmr_cosine_margin: Some(0.10),
        dense_rank: Some(2),
        lexical_rank: Some(5),
    };
    let value = serde_json::to_value(&row).expect("serialize DecisionEvidenceRow");
    assert_no_field_named(&value, "query");
    assert_no_field_named(&value, "text");
    assert_no_field_named(&value, "url");
    assert_no_field_named(&value, "heading");
    assert_no_field_named(&value, "vector");
    assert_no_field_named(&value, "credential");
}

/// Evidence sidecar rows are ordered by query ID.
#[test]
fn evidence_rows_are_ordered_by_id() {
    let mut rows = vec![
        DecisionEvidenceRow {
            id: "q2".to_string(),
            docset: "nextjs".to_string(),
            split: EvalSplit::Development,
            form: QueryForm::NaturalLanguage,
            class: QueryClass::Positive,
            answer_state: AnswerState::Confident,
            decision_reason: ReportDecisionReason::CurrentGatePass,
            top_selected_cosine: Some(0.85),
            top_selected_rrf: Some(0.03),
            pre_mmr_top_cosine: Some(0.90),
            pre_mmr_second_cosine: Some(0.80),
            pre_mmr_cosine_margin: Some(0.10),
            dense_rank: Some(2),
            lexical_rank: Some(5),
        },
        DecisionEvidenceRow {
            id: "q1".to_string(),
            docset: "nextjs".to_string(),
            split: EvalSplit::Test,
            form: QueryForm::Short,
            class: QueryClass::NearDomainNegative,
            answer_state: AnswerState::NoAnswer,
            decision_reason: ReportDecisionReason::CurrentGateReject,
            top_selected_cosine: None,
            top_selected_rrf: None,
            pre_mmr_top_cosine: None,
            pre_mmr_second_cosine: None,
            pre_mmr_cosine_margin: None,
            dense_rank: None,
            lexical_rank: None,
        },
    ];
    rows.sort_by(|a, b| a.id.cmp(&b.id));
    let value = serde_json::to_value(&rows).expect("serialize evidence rows");
    let arr = value.as_array().expect("rows is an array");
    assert_eq!(arr[0]["id"], serde_json::json!("q1"));
    assert_eq!(arr[1]["id"], serde_json::json!("q2"));
}

// ---------------------------------------------------------------------------
// C07b: calibrated evaluator-contract tests
// ---------------------------------------------------------------------------

/// The calibrated policy identifier is the exact public constant.
#[test]
fn calibrated_policy_id_is_exact() {
    assert_eq!(CALIBRATED_POLICY_ID, "calibrated-cosine-0.845-v1");
}

/// The binary gate policy identifier is preserved (historical reports).
#[test]
fn binary_gate_policy_id_is_preserved() {
    assert_eq!(
        nowdocs::eval::BINARY_GATE_POLICY_ID,
        "binary-current-gate-v1"
    );
}
