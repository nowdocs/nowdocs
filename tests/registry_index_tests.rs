//! Tests for registry catalog discovery (U3.2 / U3.3): parsing, search, and
//! the security gate that rejects indexes with disallowed download URLs.

use nowdocs::registry::{fetch_index_from, search_packages};
use std::path::PathBuf;

fn fixture_url() -> String {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let path = PathBuf::from(manifest).join("seed-crates/index.json");
    format!("file://{}", path.display())
}

#[test]
fn parses_local_index_fixture() {
    let idx = fetch_index_from(&fixture_url()).expect("parse index.json");
    assert_eq!(idx.schema_version, 1);
    assert_eq!(idx.packages.len(), 3);
    assert_eq!(idx.packages[0].docset, "nextjs");
    // fetch_index_from enforces is_allowed_registry_url on every package
    // download_url; reaching here means all URLs are on allowed domains.
}

#[test]
fn search_filters_by_name_and_description() {
    let idx = fetch_index_from(&fixture_url()).unwrap();

    // "react" matches both the `react` package (by name) and `nextjs`
    // (description "React framework for production") — substring search on
    // name + description is intentional.
    let by_name = search_packages(&idx, "react");
    assert_eq!(by_name.len(), 2);
    let names: Vec<&str> = by_name.iter().map(|p| p.docset.as_str()).collect();
    assert!(names.contains(&"react"));
    assert!(names.contains(&"nextjs"));

    let by_desc = search_packages(&idx, "progressive");
    assert_eq!(by_desc.len(), 1);
    assert_eq!(by_desc[0].docset, "vue");

    let none = search_packages(&idx, "no-such-docset");
    assert!(none.is_empty());
}

#[test]
fn rejects_index_with_disallowed_download_url() {
    let path = std::env::temp_dir().join(format!("nowdocs_bad_index_{}.json", std::process::id()));
    let json = r#"{
      "schema_version": 1,
      "generated_at": "2026-07-07T00:00:00Z",
      "packages": [
        {
          "docset": "evil",
          "version": "1.0.0",
          "license": "MIT",
          "chunk_count": 1,
          "freshness": "2026-07-07",
          "download_url": "https://evil.example.com/evil.tar",
          "sha256": "00",
          "description": "should be rejected"
        }
      ]
    }"#;
    std::fs::write(&path, json).unwrap();
    let url = format!("file://{}", path.display());
    let result = fetch_index_from(&url);
    assert!(
        result.is_err(),
        "index with disallowed download_url must be rejected"
    );
    let _ = std::fs::remove_file(&path);
}
