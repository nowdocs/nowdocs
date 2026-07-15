//! Tests for registry catalog discovery (U3.2 / U3.3): parsing, search, and
//! the security gate that rejects indexes with disallowed download URLs.

use nowdocs::registry::{fetch_index_from, search_packages};
use std::path::PathBuf;
use std::time::Duration;

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

#[test]
fn rejects_index_with_disallowed_license() {
    let path =
        std::env::temp_dir().join(format!("nowdocs_bad_license_{}.json", std::process::id()));
    let json = r#"{
      "schema_version": 1,
      "generated_at": "2026-07-07T00:00:00Z",
      "packages": [
        {
          "docset": "proprietary",
          "version": "1.0.0",
          "license": "Proprietary",
          "chunk_count": 1,
          "freshness": "2026-07-07",
          "download_url": "https://github.com/nowdocs-registry/proprietary/releases/download/proprietary-1.0.0/proprietary-1.0.0.lance.tar",
          "sha256": "0000000000000000000000000000000000000000000000000000000000000000",
          "description": "must be rejected"
        }
      ]
    }"#;
    std::fs::write(&path, json).unwrap();
    let url = format!("file://{}", path.display());
    let result = fetch_index_from(&url);
    assert!(
        result.is_err(),
        "index with disallowed license must be rejected"
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn rejects_index_with_bad_sha256() {
    let path = std::env::temp_dir().join(format!("nowdocs_bad_sha256_{}.json", std::process::id()));
    let json = r#"{
      "schema_version": 1,
      "generated_at": "2026-07-07T00:00:00Z",
      "packages": [
        {
          "docset": "badhash",
          "version": "1.0.0",
          "license": "MIT",
          "chunk_count": 1,
          "freshness": "2026-07-07",
          "download_url": "https://github.com/nowdocs-registry/badhash/releases/download/badhash-1.0.0/badhash-1.0.0.lance.tar",
          "sha256": "00",
          "description": "must be rejected"
        }
      ]
    }"#;
    std::fs::write(&path, json).unwrap();
    let url = format!("file://{}", path.display());
    let result = fetch_index_from(&url);
    assert!(
        result.is_err(),
        "index with non-64-hex sha256 must be rejected"
    );
    let _ = std::fs::remove_file(&path);
}

// A1.1 review (P1): a catalog download_url on a github /raw/ branch path must
// be rejected — package artifacts must come from a GitHub Releases download,
// not arbitrary repo content, even though the index itself is fetched from a
// /raw/ path.
#[test]
fn rejects_index_with_github_raw_download_url() {
    let path = std::env::temp_dir().join(format!("nowdocs_raw_url_{}.json", std::process::id()));
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
          "download_url": "https://github.com/nowdocs-registry/evil/raw/main/evil.tar",
          "sha256": "0000000000000000000000000000000000000000000000000000000000000000",
          "description": "raw branch path must be rejected"
        }
      ]
    }"#;
    std::fs::write(&path, json).unwrap();
    let url = format!("file://{}", path.display());
    let result = fetch_index_from(&url);
    assert!(
        result.is_err(),
        "index with a github /raw/ package download_url must be rejected"
    );
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("disallowed download_url"),
        "error must cite disallowed download_url, got: {msg}"
    );
    let _ = std::fs::remove_file(&path);
}

// A1.1 review (P1): a legit GitHub Releases download URL is still accepted by
// the stricter package gate.
#[test]
fn accepts_index_with_github_release_download_url() {
    let path = std::env::temp_dir().join(format!("nowdocs_rel_url_{}.json", std::process::id()));
    let json = r#"{
      "schema_version": 1,
      "generated_at": "2026-07-07T00:00:00Z",
      "packages": [
        {
          "docset": "nextjs",
          "version": "14.2.5",
          "license": "MIT",
          "chunk_count": 100,
          "freshness": "2026-07-07",
          "download_url": "https://github.com/nowdocs-registry/nextjs-docs/releases/download/docs-2026-07-12/nextjs-2026-07-12-ustar.tar.gz",
          "sha256": "0000000000000000000000000000000000000000000000000000000000000000",
          "description": "release artifact"
        }
      ]
    }"#;
    std::fs::write(&path, json).unwrap();
    let url = format!("file://{}", path.display());
    let idx = fetch_index_from(&url).expect("release download_url must be accepted");
    assert_eq!(idx.packages.len(), 1);
    assert_eq!(idx.packages[0].docset, "nextjs");
    let _ = std::fs::remove_file(&path);
}

// ---- Update-index reader tests ----

#[test]
fn update_reader_validates_fixture_index_without_writing_it_to_disk() {
    let idx =
        nowdocs::registry::fetch_index_for_update_from(&fixture_url(), Duration::from_millis(50))
            .unwrap();
    assert_eq!(idx.packages.len(), 3);
}

#[test]
fn update_reader_rejects_disallowed_redirects() {
    let result = nowdocs::registry::validate_update_index_redirect(
        "https://github.com/nowdocs-registry/registry-index/raw/main/index.json",
        "https://evil.example/index.json",
    );
    assert!(result.is_err(), "redirect to evil.example must be rejected");
}
