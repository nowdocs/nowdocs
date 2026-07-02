use nowdocs::chunker::{chunk_markdown, default_config, ChunkType};
use nowdocs::token::count_tokens;

#[test]
fn empty_input_yields_no_chunks() {
    let cfg = default_config();
    assert!(chunk_markdown("", &cfg).is_empty());
}

#[test]
fn heading_path_tracks_stack() {
    let cfg = default_config();
    let md = "# Title\n\nintro paragraph one.\n\n## Sub\n\nparagraph under sub.\n";
    let chunks = chunk_markdown(md, &cfg);
    assert!(!chunks.is_empty(), "should produce chunks");
    // The "Title > Sub" section must appear on at least one chunk.
    assert!(
        chunks.iter().any(|c| c.heading_path == "Title > Sub"),
        "no chunk carried heading_path 'Title > Sub': {:?}",
        chunks.iter().map(|c| &c.heading_path).collect::<Vec<_>>()
    );
    // Top-level intro chunks carry just "Title".
    assert!(chunks.iter().any(|c| c.heading_path == "Title"));
}

#[test]
fn chunks_are_sequentially_indexed_from_zero() {
    let cfg = default_config();
    let md = "# H1\n\npara one.\n\n## H2\n\npara two.\n\n## H3\n\npara three.\n";
    let chunks = chunk_markdown(md, &cfg);
    for (i, c) in chunks.iter().enumerate() {
        assert_eq!(c.idx, i as u32, "idx mismatch at {}", i);
    }
}

#[test]
fn fenced_code_block_stays_in_one_chunk_even_when_over_target() {
    let cfg = default_config(); // target 384, max 512
                                // Build a code block well over target_tokens (384) but each line short.
    let mut code = String::from("```rust\n");
    for _ in 0..500 {
        code.push_str("let x = 42; // a line\n");
    }
    code.push_str("```\n");
    let md = format!("# Code Section\n\n{code}\n");

    let chunks = chunk_markdown(&md, &cfg);
    let code_chunks: Vec<_> = chunks
        .iter()
        .filter(|c| c.chunk_type == ChunkType::Code)
        .collect();

    assert_eq!(
        code_chunks.len(),
        1,
        "fenced code block must stay in ONE chunk, got {}: {:?}",
        code_chunks.len(),
        code_chunks.iter().map(|c| c.idx).collect::<Vec<_>>()
    );
    // And it legitimately exceeds target_tokens (proving it wasn't split to fit).
    let code_text_tokens = count_tokens(&code_chunks[0].text);
    assert!(
        code_text_tokens > cfg.target_tokens as usize,
        "code chunk should exceed target ({}), got {}",
        cfg.target_tokens,
        code_text_tokens
    );
}

#[test]
fn prose_chunks_respect_max_tokens() {
    let cfg = default_config(); // max 512
                                // A long prose paragraph with no code — must be split to stay ≤ max_tokens.
    let mut para = String::new();
    for i in 0..2000 {
        para.push_str(&format!(
            "Sentence number {} about how rust works well. ",
            i
        ));
    }
    let md = format!("# Prose\n\n{para}\n");

    let chunks = chunk_markdown(&md, &cfg);
    let prose_chunks: Vec<_> = chunks
        .iter()
        .filter(|c| c.chunk_type == ChunkType::Info)
        .collect();
    assert!(
        prose_chunks.len() > 1,
        "long prose should be split into multiple chunks"
    );
    for c in &prose_chunks {
        let n = count_tokens(&c.text);
        assert!(
            n <= cfg.max_tokens as usize,
            "prose chunk {} exceeds max_tokens ({} > {})",
            c.idx,
            n,
            cfg.max_tokens
        );
    }
}

#[test]
fn code_chunk_classified_as_code_prose_as_info() {
    let cfg = default_config();
    let md = "# Mixed\n\nsome prose line.\n\n```js\nconst a = 1;\n```\n";
    let chunks = chunk_markdown(md, &cfg);
    assert!(chunks.iter().any(|c| c.chunk_type == ChunkType::Code));
    assert!(chunks.iter().any(|c| c.chunk_type == ChunkType::Info));
}
