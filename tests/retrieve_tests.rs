use nowdocs::chunker::ChunkType;
use nowdocs::retrieve::{ResultChunk, SearchResult};

#[test]
fn test_search_smoke() {
    let _ = std::hint::black_box((ResultChunk {
        chunk_idx: 0,
        heading_path: "H".into(),
        source_url: "a.md".into(),
        api_version: None,
        chunk_type: ChunkType::Info,
        text: "hello".into(),
    }, SearchResult {
        chunks: vec![],
        tokens_returned: 0,
        truncated: false,
    }));
}

#[test]
fn test_search_rejects_invalid_inputs() {
    use nowdocs::retrieve::search;
    unsafe { std::env::set_var("XDG_CACHE_HOME", tempfile::tempdir().unwrap().path()) };
    assert!(search("../bad", "query", None, None).is_err());
    assert!(search("valid", "", None, None).is_err());
}

#[test]
#[ignore = "needs real embedder (~66MB download, ~30s)"]
fn test_search_end_to_end() {
    use nowdocs::ingest::ingest_dir;
    use nowdocs::retrieve::search;
    use std::fs;

    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };

    fs::write(dir.path().join("a.md"), "# Auth\n\nUse token zzzretrieve_xyz to authenticate.\n").unwrap();
    fs::write(dir.path().join("b.md"), "# Config\n\nSet timeout to 30s.\n").unwrap();

    let stats = ingest_dir(dir.path(), "retrieve_e2e").unwrap();
    assert!(stats.chunks >= 2);

    let result = search("retrieve_e2e", "zzzretrieve_xyz", Some(4000), Some(5)).unwrap();
    assert!(!result.chunks.is_empty(), "should return at least one chunk");
    assert!(
        result.chunks.iter().any(|c| c.text.contains("zzzretrieve_xyz")),
        "recalled chunk must contain the unique keyword"
    );
    assert!(result.tokens_returned <= 4000, "tokens must fit budget");
    assert!(result.chunks.windows(2).all(|w| w[0].chunk_idx < w[1].chunk_idx), "chunks sorted by idx");
}
