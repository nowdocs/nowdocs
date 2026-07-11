use nowdocs::manifest::{parse_manifest, schema_compatibility, validate, SchemaCompatibility};

const VALID: &str = r#"{
  "docset":"nextjs","doc_version":"15.1.0","nowdocs_schema_version":1,
  "embedder":{"model_id":"jinaai/jina-embeddings-v2-small-en","model_version":"1.0.2",
    "model_revision":"abc","model_sha256":"def","vector_dim":512,"engine":"candle","dtype":"f16"},
  "retrieval":{"tokenizer":"default","chunk_size_tokens":384,"window_tokens":2048},
  "source":{"entry_url":"https://nextjs.org/docs","source_url":"https://github.com/vercel/next.js",
    "scraped_at":"2026-06-28T10:00:00Z","chunk_count":100},
  "legal":{"license":"MIT","copyright_holder":"Vercel Inc.","attribution":"Copyright (c) Vercel Inc. — MIT"},
  "refresh_strategy":{"tier":"top100","auto_days":1}
}"#;

#[test]
fn parses_valid_manifest() {
    assert!(parse_manifest(VALID).is_ok());
}

#[test]
fn parses_and_roundtrips_fields() {
    let m = parse_manifest(VALID).unwrap();
    assert_eq!(m.docset, "nextjs");
    assert_eq!(m.embedder.vector_dim, 512);
    assert_eq!(m.retrieval.tokenizer, "default");
    assert_eq!(m.legal.license, "MIT");
    assert!(validate(&m).is_ok());
}

#[test]
fn rejects_unknown_schema_version_with_rebuild_hint() {
    let mut v: serde_json::Value = serde_json::from_str(VALID).unwrap();
    v["nowdocs_schema_version"] = serde_json::json!(999);
    let err = validate(&serde_json::from_value(v).unwrap())
        .unwrap_err()
        .to_string();
    assert!(err.contains("newer than this nowdocs binary supports"));
    assert!(err.contains("nowdocs rebuild nextjs"));
}

#[test]
fn rejects_non_allowlisted_license() {
    let mut v: serde_json::Value = serde_json::from_str(VALID).unwrap();
    v["legal"]["license"] = serde_json::json!("proprietary");
    let m: nowdocs::manifest::Manifest = serde_json::from_value(v).unwrap();
    assert!(validate(&m).is_err());
}

#[test]
fn requires_attribution_for_ccby() {
    let mut v: serde_json::Value = serde_json::from_str(VALID).unwrap();
    v["legal"]["license"] = serde_json::json!("CC-BY-4.0");
    v["legal"]["attribution"] = serde_json::json!("");
    let m: nowdocs::manifest::Manifest = serde_json::from_value(v).unwrap();
    assert!(validate(&m).is_err());
}

// A1.1 §3.8 spec-named aliases for the license/attribution invariants. M12
// (`validate_manifest_for_docset`) deliberately does NOT re-check these — they
// live in `manifest::validate` — so the spec names are asserted against
// `validate` directly, which is where the allowlist / CC-BY rules are enforced.

#[test]
fn rejects_manifest_with_disallowed_license() {
    let mut v: serde_json::Value = serde_json::from_str(VALID).unwrap();
    v["legal"]["license"] = serde_json::json!("proprietary");
    let m: nowdocs::manifest::Manifest = serde_json::from_value(v).unwrap();
    assert!(
        validate(&m).is_err(),
        "a license outside the MIT/Apache-2.0/CC-BY-4.0 allowlist must be rejected"
    );
}

#[test]
fn rejects_ccby4_without_attribution() {
    let mut v: serde_json::Value = serde_json::from_str(VALID).unwrap();
    v["legal"]["license"] = serde_json::json!("CC-BY-4.0");
    v["legal"]["attribution"] = serde_json::json!("");
    let m: nowdocs::manifest::Manifest = serde_json::from_value(v).unwrap();
    assert!(
        validate(&m).is_err(),
        "CC-BY-4.0 without attribution must be rejected"
    );
}

#[test]
fn rejects_wrong_embedder_model() {
    let mut v: serde_json::Value = serde_json::from_str(VALID).unwrap();
    v["embedder"]["model_id"] = serde_json::json!("some/other-model");
    let m: nowdocs::manifest::Manifest = serde_json::from_value(v).unwrap();
    assert!(validate(&m).is_err());
}

#[test]
fn rejects_non_default_tokenizer() {
    let mut v: serde_json::Value = serde_json::from_str(VALID).unwrap();
    v["retrieval"]["tokenizer"] = serde_json::json!("lindera");
    let m: nowdocs::manifest::Manifest = serde_json::from_value(v).unwrap();
    assert!(validate(&m).is_err());
}

#[test]
fn reports_schema_compatibility_without_bailing() {
    assert_eq!(schema_compatibility(1), SchemaCompatibility::Current);
    assert_eq!(
        schema_compatibility(0),
        SchemaCompatibility::Older {
            found: 0,
            current: 1
        }
    );
    assert_eq!(
        schema_compatibility(2),
        SchemaCompatibility::Newer {
            found: 2,
            current: 1
        }
    );
}

// ---- M12: install-context business invariants (validate_manifest_for_docset) ----
// These complement `validate` (self-contained schema/model/license) with checks
// that only make sense relative to an install name. License / attribution are
// intentionally NOT re-checked here — `validate` already covers them.

use nowdocs::manifest::validate_manifest_for_docset;

#[test]
fn rejects_manifest_with_wrong_docset() {
    let m = parse_manifest(VALID).unwrap();
    assert!(
        validate_manifest_for_docset(&m, "some-other-docset").is_err(),
        "docset identity mismatch must be rejected"
    );
}

#[test]
fn rejects_manifest_with_zero_chunk_count() {
    let mut v: serde_json::Value = serde_json::from_str(VALID).unwrap();
    v["source"]["chunk_count"] = serde_json::json!(0);
    let m: nowdocs::manifest::Manifest = serde_json::from_value(v).unwrap();
    assert!(
        validate_manifest_for_docset(&m, "nextjs").is_err(),
        "chunk_count == 0 must be rejected"
    );
}

#[test]
fn rejects_manifest_with_no_source_urls() {
    let mut v: serde_json::Value = serde_json::from_str(VALID).unwrap();
    v["source"]["source_url"] = serde_json::json!("");
    v["source"]["entry_url"] = serde_json::json!("");
    let m: nowdocs::manifest::Manifest = serde_json::from_value(v).unwrap();
    assert!(
        validate_manifest_for_docset(&m, "nextjs").is_err(),
        "both source_url and entry_url empty must be rejected"
    );
}

#[test]
fn accepts_valid_manifest_for_docset() {
    let m = parse_manifest(VALID).unwrap();
    assert!(
        validate_manifest_for_docset(&m, "nextjs").is_ok(),
        "valid manifest with matching docset must pass: {:?}",
        validate_manifest_for_docset(&m, "nextjs").err()
    );
}

#[test]
fn accepts_manifest_with_only_entry_url() {
    // source_url empty but entry_url present → still traceable, must pass.
    let mut v: serde_json::Value = serde_json::from_str(VALID).unwrap();
    v["source"]["source_url"] = serde_json::json!("");
    let m: nowdocs::manifest::Manifest = serde_json::from_value(v).unwrap();
    assert!(validate_manifest_for_docset(&m, "nextjs").is_ok());
}
