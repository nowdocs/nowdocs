use nowdocs::cache;
use nowdocs::embedder::Embedder;
use nowdocs::ingest::ingest_dir;
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

    let stats = ingest_dir(dir.path(), "test_ingest").unwrap();
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
    assert!(ingest_dir(dir.path(), "../bad").is_err());      // path traversal
    assert!(ingest_dir(dir.path(), "BadDocset").is_err());  // uppercase
}

#[test]
fn test_ingest_empty_dir() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };
    let stats = ingest_dir(dir.path(), "empty_ds").unwrap();
    assert_eq!(stats.files, 0);
    assert_eq!(stats.chunks, 0);
    let m = manifest::parse_manifest(
        &fs::read_to_string(cache::manifest_path("empty_ds")).unwrap()).unwrap();
    manifest::validate(&m).unwrap();
    assert_eq!(m.source.chunk_count, 0);
}
