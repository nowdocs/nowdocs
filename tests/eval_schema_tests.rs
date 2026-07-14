//! Schema-level tests for the versioned labeled-query evaluation foundation:
//! target matching, suite validation, fixture loading, and `AnswerState`
//! serialization. Pure: no embedder, no I/O beyond the committed fixture.

use nowdocs::confidence::AnswerState;
use std::collections::HashSet;

use nowdocs::eval::{
    hit_matches_target, load_eval_file, validate_suite, EvalQuery, EvalSplit, QueryClass,
    QueryForm, RelevanceTarget,
};

/// String name of an `EvalSplit` variant for readable assertion messages.
fn split_name(split: &EvalSplit) -> &'static str {
    match split {
        EvalSplit::Development => "development",
        EvalSplit::Test => "test",
    }
}
use nowdocs::retrieve::ResultChunk;

fn hit(source_url: &str, heading_path: &str) -> ResultChunk {
    ResultChunk {
        chunk_idx: 7,
        heading_path: heading_path.into(),
        source_url: source_url.into(),
        api_version: None,
        chunk_type: nowdocs::chunker::ChunkType::Info,
        text: "matcher docs".into(),
        score: Some(0.03),
    }
}

fn positive_query(id: &str, docset: &str, split: EvalSplit, family: &str) -> EvalQuery {
    EvalQuery {
        id: id.into(),
        docset: docset.into(),
        query: "how do I configure the matcher".into(),
        split,
        intent_family: family.into(),
        query_form: QueryForm::NaturalLanguage,
        query_class: QueryClass::Positive,
        targets: vec![RelevanceTarget {
            source_url: "proxy.md".into(),
            heading_path_prefix: None,
            grade: 2,
        }],
    }
}

fn negative_query(id: &str, docset: &str, split: EvalSplit, family: &str) -> EvalQuery {
    EvalQuery {
        id: id.into(),
        docset: docset.into(),
        query: "vue composition api".into(),
        split,
        intent_family: family.into(),
        query_form: QueryForm::Short,
        query_class: QueryClass::CrossDomainNegative,
        targets: vec![],
    }
}

#[test]
fn heading_target_requires_exact_or_descendant_segment() {
    let hit = ResultChunk {
        chunk_idx: 7,
        heading_path: "Exports > Matcher > Negative matching".into(),
        source_url: "01-app/03-api-reference/03-file-conventions/proxy.md".into(),
        api_version: None,
        chunk_type: nowdocs::chunker::ChunkType::Info,
        text: "matcher docs".into(),
        score: Some(0.03),
    };
    let target = RelevanceTarget {
        source_url: hit.source_url.clone(),
        heading_path_prefix: Some("Exports > Matcher".into()),
        grade: 2,
    };
    assert!(hit_matches_target(&hit, &target));
    assert!(!hit_matches_target(
        &hit,
        &RelevanceTarget {
            heading_path_prefix: Some("Exports > Match".into()),
            ..target
        }
    ));
}

#[test]
fn heading_target_matches_exact_normalized_heading() {
    let h = hit("proxy.md", "Exports > Matcher");
    let target = RelevanceTarget {
        source_url: "proxy.md".into(),
        // Unnormalized prefix (extra `##` marks and spacing) must normalize to
        // the same heading.
        heading_path_prefix: Some("## Exports>  Matcher".into()),
        grade: 2,
    };
    assert!(hit_matches_target(&h, &target));
}

#[test]
fn target_requires_exact_source_url_equality() {
    let h = hit("proxy.md", "Exports > Matcher");
    let target = RelevanceTarget {
        source_url: "other.md".into(),
        heading_path_prefix: Some("Exports > Matcher".into()),
        grade: 2,
    };
    assert!(
        !hit_matches_target(&h, &target),
        "heading match must not compensate for a different source_url"
    );
}

#[test]
fn target_without_heading_prefix_matches_on_source_url_alone() {
    let h = hit("proxy.md", "Exports > Anything");
    let target = RelevanceTarget {
        source_url: "proxy.md".into(),
        heading_path_prefix: None,
        grade: 1,
    };
    assert!(hit_matches_target(&h, &target));
}

#[test]
fn validate_suite_rejects_duplicate_query_ids() {
    let suite = vec![
        positive_query("q1", "nextjs", EvalSplit::Development, "matcher"),
        positive_query("q1", "nextjs", EvalSplit::Development, "rewrites"),
    ];
    assert!(validate_suite(&suite).is_err());
}

#[test]
fn validate_suite_rejects_intent_family_in_both_splits() {
    let suite = vec![
        positive_query("q1", "nextjs", EvalSplit::Development, "matcher"),
        positive_query("q2", "nextjs", EvalSplit::Test, "matcher"),
    ];
    assert!(
        validate_suite(&suite).is_err(),
        "same (docset, intent_family) in both splits must be rejected"
    );
}

#[test]
fn validate_suite_allows_same_family_in_different_docsets() {
    let suite = vec![
        positive_query("q1", "nextjs", EvalSplit::Development, "matcher"),
        positive_query("q2", "react", EvalSplit::Test, "matcher"),
    ];
    assert!(validate_suite(&suite).is_ok());
}

#[test]
fn validate_suite_rejects_grade_outside_range() {
    for grade in [0u8, 3] {
        let mut q = positive_query("q1", "nextjs", EvalSplit::Development, "matcher");
        q.targets[0].grade = grade;
        assert!(
            validate_suite(&[q]).is_err(),
            "grade {grade} must be rejected"
        );
    }
}

#[test]
fn validate_suite_rejects_positive_without_targets() {
    let mut q = positive_query("q1", "nextjs", EvalSplit::Development, "matcher");
    q.targets.clear();
    assert!(validate_suite(&[q]).is_err());
}

#[test]
fn validate_suite_rejects_negative_with_targets() {
    for class in [
        QueryClass::NearDomainNegative,
        QueryClass::CrossDomainNegative,
    ] {
        let mut q = negative_query("q1", "nextjs", EvalSplit::Development, "neg");
        q.query_class = class;
        q.targets.push(RelevanceTarget {
            source_url: "proxy.md".into(),
            heading_path_prefix: None,
            grade: 1,
        });
        assert!(
            validate_suite(&[q]).is_err(),
            "negative query with a target must be rejected"
        );
    }
}

#[test]
fn validate_suite_rejects_empty_fields() {
    let cases: Vec<EvalQuery> = vec![
        {
            let mut q = positive_query("q1", "nextjs", EvalSplit::Development, "matcher");
            q.id.clear();
            q
        },
        {
            let mut q = positive_query("q1", "nextjs", EvalSplit::Development, "matcher");
            q.docset.clear();
            q
        },
        {
            let mut q = positive_query("q1", "nextjs", EvalSplit::Development, "matcher");
            q.query.clear();
            q
        },
        {
            let mut q = positive_query("q1", "nextjs", EvalSplit::Development, "matcher");
            q.intent_family.clear();
            q
        },
    ];
    for q in cases {
        assert!(
            validate_suite(std::slice::from_ref(&q)).is_err(),
            "rejected: {q:?}"
        );
    }
}

#[test]
fn validate_suite_accepts_multiple_targets_for_positive() {
    let mut q = positive_query("q1", "nextjs", EvalSplit::Development, "matcher");
    q.targets.push(RelevanceTarget {
        source_url: "rewrites.md".into(),
        heading_path_prefix: Some("Exports".into()),
        grade: 1,
    });
    assert!(validate_suite(&[q]).is_ok());
}

#[test]
fn schema_smoke_fixture_loads_and_validates() {
    let raw = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/eval/schema-smoke.json"
    ))
    .expect("read schema-smoke.json");
    let suite: Vec<EvalQuery> =
        serde_json::from_str(&raw).expect("schema-smoke.json parses as Vec<EvalQuery>");
    assert!(
        suite.len() >= 4,
        "smoke fixture should exercise several query shapes, got {}",
        suite.len()
    );
    validate_suite(&suite).expect("schema-smoke.json must pass suite validation");
}

#[test]
fn eval_query_json_round_trip() {
    let q = positive_query("q1", "nextjs", EvalSplit::Development, "matcher");
    let json = serde_json::to_string(&q).expect("serialize EvalQuery");
    let back: EvalQuery = serde_json::from_str(&json).expect("deserialize EvalQuery");
    assert_eq!(q, back);
    assert!(
        json.contains("\"query_class\":\"positive\""),
        "enums must serialize snake_case, got {json}"
    );
}

#[test]
fn answer_state_serializes_snake_case() {
    let cases = [
        (AnswerState::Confident, "\"confident\""),
        (AnswerState::Borderline, "\"borderline\""),
        (AnswerState::NoAnswer, "\"no_answer\""),
    ];
    for (state, expected) in cases {
        let json = serde_json::to_string(&state).expect("serialize AnswerState");
        assert_eq!(json, expected);
        let back: AnswerState = serde_json::from_str(&json).expect("deserialize AnswerState");
        assert_eq!(back, state);
    }
}

// ---------------------------------------------------------------------------
// C10: real-suite composition gates
// ---------------------------------------------------------------------------

/// Load each real eval suite independently via `load_eval_file` and assert
/// minimum family counts. Counts are by distinct `intent_family`, not query
/// rows. `schema-smoke.json` is deliberately excluded — it shares a Next.js
/// intent family and would inflate counts if loaded through the directory
/// loader.
#[test]
fn real_suites_meet_minimum_family_counts() {
    let fixtures_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/eval");
    let suites = ["nextjs.json", "react.json", "vue.json"];

    for suite_file in suites {
        let path = format!("{fixtures_dir}/{suite_file}");
        let queries = load_eval_file(&path).expect("load_eval_file must succeed");

        // 1. All records use the corresponding docset name.
        let expected_docset = suite_file.trim_end_matches(".json");
        for q in &queries {
            assert_eq!(
                q.docset, expected_docset,
                "{suite_file}: query {} has docset {}, expected {expected_docset}",
                q.id, q.docset
            );
        }

        // 2. validate_suite succeeds on the individual suite.
        validate_suite(&queries).expect("{suite_file}: validate_suite must succeed");

        // 3. Per-split minimum family counts.
        for split in &[EvalSplit::Development, EvalSplit::Test] {
            let sn = split_name(split);
            let pos_families: HashSet<&str> = queries
                .iter()
                .filter(|q| q.split == *split && q.query_class == QueryClass::Positive)
                .map(|q| q.intent_family.as_str())
                .collect();
            let neg_families: HashSet<&str> = queries
                .iter()
                .filter(|q| {
                    q.split == *split
                        && matches!(
                            q.query_class,
                            QueryClass::NearDomainNegative | QueryClass::CrossDomainNegative
                        )
                })
                .map(|q| q.intent_family.as_str())
                .collect();
            let near_neg_families: HashSet<&str> = queries
                .iter()
                .filter(|q| q.split == *split && q.query_class == QueryClass::NearDomainNegative)
                .map(|q| q.intent_family.as_str())
                .collect();

            assert!(
                pos_families.len() >= 20,
                "{suite_file} {sn}: expected >= 20 distinct positive intent families, got {}",
                pos_families.len()
            );
            assert!(
                neg_families.len() >= 15,
                "{suite_file} {sn}: expected >= 15 distinct negative intent families, got {}",
                neg_families.len()
            );

            // 4. At least 50% of distinct negative families are near_domain_negative.
            let near_share = near_neg_families.len() as f32 / neg_families.len() as f32;
            assert!(
                near_share >= 0.50,
                "{suite_file} {sn}: near_domain_negative share {:.0}% < 50%",
                near_share * 100.0
            );
        }

        // 5. (docset, intent_family) appears in only one split.
        // validate_suite already enforces this; we exercise it through the
        // real suites by loading each independently and calling validate_suite
        // above.
    }
}

#[test]
fn real_suite_targets_use_installed_corpus_source_urls() {
    let fixtures_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/eval");
    let suites = [
        (
            "nextjs.json",
            "https://github.com/vercel/next.js/tree/canary/docs",
        ),
        (
            "react.json",
            "https://github.com/reactjs/react.dev/tree/main/src/content",
        ),
        ("vue.json", "https://github.com/vuejs/docs/tree/main/src"),
    ];

    for (suite_file, source_url_base) in suites {
        let path = format!("{fixtures_dir}/{suite_file}");
        let queries = load_eval_file(&path).expect("load_eval_file must succeed");
        let required_prefix = format!("{source_url_base}/");

        for target in queries.iter().flat_map(|query| &query.targets) {
            assert!(
                target.source_url.starts_with(&required_prefix),
                "{suite_file}: target URL {:?} must start with {:?}",
                target.source_url,
                required_prefix
            );
        }
    }
}
