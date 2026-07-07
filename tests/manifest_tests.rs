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
