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
fn chunker_splits_oversized_code_block() {
    let cfg = default_config(); // max 512
    let code = big_rust_block(); // well over max_tokens
    let md = format!("# Code Section\n\n{code}\n");

    let chunks = chunk_markdown(&md, &cfg);
    let code_chunks: Vec<_> = chunks
        .iter()
        .filter(|c| c.chunk_type == ChunkType::Code)
        .collect();

    assert!(
        code_chunks.len() > 1,
        "oversized code block must be split into multiple chunks, got {}",
        code_chunks.len()
    );
    for c in &code_chunks {
        let n = count_tokens(&c.text);
        assert!(
            n <= cfg.max_tokens as usize,
            "code sub-chunk {} exceeds max_tokens ({} > {})",
            c.idx,
            n,
            cfg.max_tokens
        );
    }
}

#[test]
fn chunker_preserves_fence_markers_on_split() {
    let cfg = default_config();
    let code = big_rust_block();
    let md = format!("# Code Section\n\n{code}\n");

    let chunks = chunk_markdown(&md, &cfg);
    let code_chunks: Vec<_> = chunks
        .iter()
        .filter(|c| c.chunk_type == ChunkType::Code)
        .collect();

    assert!(
        code_chunks.len() > 1,
        "expected split to test fence preservation, got {}",
        code_chunks.len()
    );
    for c in &code_chunks {
        let prefix = format!("## {}\n\n", c.heading_path);
        let body = c.text.strip_prefix(&prefix).unwrap_or(&c.text);
        assert!(
            body.trim_start().starts_with("```"),
            "sub-chunk {} must start with opening fence, got: {:?}",
            c.idx,
            &body[..body.len().min(40)]
        );
        assert!(
            body.contains("\n```"),
            "sub-chunk {} must contain a closing fence",
            c.idx
        );
    }
}

#[test]
fn chunker_appends_part_suffix_on_split() {
    let cfg = default_config();
    let code = big_rust_block();
    let md = format!("# Code Section\n\n{code}\n");

    let chunks = chunk_markdown(&md, &cfg);
    let code_chunks: Vec<_> = chunks
        .iter()
        .filter(|c| c.chunk_type == ChunkType::Code)
        .collect();

    assert!(code_chunks.len() > 1, "expected split to test part suffix");
    for (i, c) in code_chunks.iter().enumerate() {
        let expected = format!("Code Section (part {})", i + 1);
        assert_eq!(
            c.heading_path, expected,
            "sub-chunk heading must carry part suffix"
        );
    }
}

fn big_rust_block() -> String {
    // Many small functions so the chunker can split on `fn ` boundaries.
    let mut code = String::from("```rust\n");
    for f in 0..60 {
        code.push_str(&format!("fn func_{f}() {{\n"));
        for _ in 0..12 {
            code.push_str("    let value = 42; // compute something here\n");
        }
        code.push_str("}\n\n");
    }
    code.push_str("```\n");
    code
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
