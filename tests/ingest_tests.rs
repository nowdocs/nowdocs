use nowdocs::cache;
use nowdocs::embedder::Embedder;
use nowdocs::ingest::{ingest_dir, IngestMeta};
use nowdocs::manifest;
use nowdocs::store::Store;
use std::fs;

#[test]
#[ignore = "needs real embedder (~66MB download, ~30s)"]
fn test_ingest_end_to_end() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    fs::write(dir.path().join("intro.md"), "# Intro\n\nGeneral setup notes.\n").unwrap();
    fs::write(dir.path().join("api.md"),
        "# API\n\n## Auth\n\nAuthentication uses zzzunique_ingest_token for bearer flow.\n").unwrap();

    let stats = ingest_dir(dir.path(), "test_ingest", &IngestMeta::default()).unwrap();
    assert_eq!(stats.files, 2);
    assert!(stats.chunks >= 2);

    // manifest written + validates
    let m = manifest::parse_manifest(&fs::read_to_string(cache::manifest_path("test_ingest")).unwrap()).unwrap();
    manifest::validate(&m).unwrap();
    assert_eq!(m.source.chunk_count, stats.chunks);
    assert_eq!(m.embedder.model_id, "jinaai/jina-embeddings-v2-small-en");

    // search recalls the unique-keyword chunk (real embed query vector + FTS BM25)
    let emb = Embedder::load().unwrap();
    let qv = emb.embed("zzzunique_ingest_token").unwrap();
    let store = Store::open("test_ingest").unwrap();
    let hits = store.hybrid_search(&qv, "zzzunique_ingest_token", 5).unwrap();
    assert!(hits.iter().any(|h| h.text.contains("zzzunique_ingest_token")),
        "unique-keyword chunk should be recalled");
}

#[test]
fn test_ingest_rejects_bad_docset() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };
    fs::write(dir.path().join("a.md"), "# A\n\ntext\n").unwrap();
    assert!(ingest_dir(dir.path(), "../bad", &IngestMeta::default()).is_err());      // path traversal
    assert!(ingest_dir(dir.path(), "BadDocset", &IngestMeta::default()).is_err());  // uppercase
}

#[test]
fn test_ingest_empty_dir() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };
    let stats = ingest_dir(dir.path(), "empty_ds", &IngestMeta::default()).unwrap();
    assert_eq!(stats.files, 0);
    assert_eq!(stats.chunks, 0);
    let m = manifest::parse_manifest(
        &fs::read_to_string(cache::manifest_path("empty_ds")).unwrap()).unwrap();
    manifest::validate(&m).unwrap();
    assert_eq!(m.source.chunk_count, 0);
}

// ---- legal/source metadata flags (absorbs patch_manifest.py) ----

#[test]
fn test_ingest_default_meta_uses_mit_and_today_scraped_at() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };
    let stats = ingest_dir(dir.path(), "def_meta_ds", &IngestMeta::default()).unwrap();
    assert_eq!(stats.files, 0);

    let m = manifest::parse_manifest(
        &fs::read_to_string(cache::manifest_path("def_meta_ds")).unwrap()).unwrap();
    manifest::validate(&m).unwrap();
    // default license is MIT (backward-compatible with the pre-flag behavior)
    assert_eq!(m.legal.license, "MIT");
    assert_eq!(m.legal.copyright_holder, "");
    assert_eq!(m.legal.attribution, "");
    // scraped_at is auto-filled with today's date as YYYY-MM-DD (no chrono dep)
    assert_eq!(m.source.scraped_at.len(), 10, "scraped_at must be YYYY-MM-DD, got: {}", m.source.scraped_at);
    assert_eq!(m.source.scraped_at.chars().nth(4), Some('-'));
    assert_eq!(m.source.scraped_at.chars().nth(7), Some('-'));
}

#[test]
fn test_ingest_cc_by_with_attribution_persists_legal_and_source() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };
    let meta = IngestMeta {
        license: "CC-BY-4.0".to_string(),
        copyright_holder: "Meta Platforms, Inc.".to_string(),
        attribution: "React documentation by Meta, licensed CC BY 4.0.".to_string(),
        source_url: "https://github.com/reactjs/react.dev".to_string(),
        entry_url: "https://react.dev".to_string(),
    };
    ingest_dir(dir.path(), "ccb_ds", &meta).unwrap();

    let m = manifest::parse_manifest(
        &fs::read_to_string(cache::manifest_path("ccb_ds")).unwrap()).unwrap();
    manifest::validate(&m).unwrap();
    assert_eq!(m.legal.license, "CC-BY-4.0");
    assert_eq!(m.legal.copyright_holder, "Meta Platforms, Inc.");
    assert_eq!(m.legal.attribution, "React documentation by Meta, licensed CC BY 4.0.");
    assert_eq!(m.source.source_url, "https://github.com/reactjs/react.dev");
    assert_eq!(m.source.entry_url, "https://react.dev");
    assert_eq!(m.source.scraped_at.len(), 10);
}

#[test]
fn test_ingest_cc_by_without_attribution_fails_validation() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };
    let meta = IngestMeta {
        license: "CC-BY-4.0".to_string(),
        ..IngestMeta::default()
    };
    let err = ingest_dir(dir.path(), "ccb_fail_ds", &meta).unwrap_err();
    let msg = format!("{err:#}");
    assert!(
        msg.contains("attribution"),
        "CC-BY-4.0 without attribution must fail validation, got: {msg}"
    );
    // Validation must happen BEFORE the store is opened, so a failed ingest
    // leaves no orphan .lance dir and no manifest for list-installed to pick up.
    assert!(
        !cache::db_path("ccb_fail_ds").exists(),
        "failed ingest must not leave an orphan .lance dir, got: {}",
        cache::db_path("ccb_fail_ds").display()
    );
    assert!(
        !cache::manifest_path("ccb_fail_ds").exists(),
        "failed ingest must not leave a manifest"
    );
}
