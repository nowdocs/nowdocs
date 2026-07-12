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
fn heading_path_normalizes_scraped_markers_and_empty_segments() {
    let cfg = default_config();
    let chunks = chunk_markdown(
        "# > > Exports > Matcher\n\nmatcher configuration.\n# API > ## Config\n\napi configuration.\n",
        &cfg,
    );
    assert!(chunks
        .iter()
        .any(|chunk| chunk.heading_path == "Exports > Matcher"));
    assert!(chunks
        .iter()
        .any(|chunk| chunk.heading_path == "API > Config"));
    assert!(chunks
        .iter()
        .all(|chunk| !chunk.heading_path.contains("> >")));
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

// --- A1.2 backlog: chunker torture fixtures (fence/heading edge cases) ---

#[test]
fn unterminated_fence_becomes_single_code_chunk() {
    let cfg = default_config();
    let md = "# API\n\n```rust\nlet x = 1;\nlet y = 2;\n";
    let chunks = chunk_markdown(md, &cfg);
    assert!(
        chunks.iter().any(|c| c.chunk_type == ChunkType::Code),
        "unterminated fence should still emit a Code chunk"
    );
    // Must not panic and must still respect the token budget.
    for c in &chunks {
        assert!(
            count_tokens(&c.text) <= cfg.max_tokens as usize,
            "chunk {} exceeds max_tokens",
            c.idx
        );
    }
}

#[test]
fn fence_marker_length_variations_recognized() {
    let cfg = default_config();
    let md = "# Note\n\n~~~~\ncontent\n~~~~\n\n`````\nmore\n`````\n";
    let chunks = chunk_markdown(md, &cfg);
    let code_chunks: Vec<_> = chunks
        .iter()
        .filter(|c| c.chunk_type == ChunkType::Code)
        .collect();
    assert_eq!(
        code_chunks.len(),
        2,
        "both ~~~~ and ````` fences should produce code chunks"
    );
}

#[test]
fn heading_stack_skips_levels_without_panic() {
    let cfg = default_config();
    let md = "# H1\n\n## H2\n\n#### H4\n\ntext under h4.\n";
    let chunks = chunk_markdown(md, &cfg);
    let h4_chunks: Vec<_> = chunks
        .iter()
        .filter(|c| c.heading_path.contains("H4"))
        .collect();
    assert!(!h4_chunks.is_empty(), "H4 chunk must exist");
    assert!(
        h4_chunks
            .iter()
            .any(|c| c.heading_path.starts_with("H1 > H2")),
        "skipped level must not lose H1/H2 ancestry, got {:?}",
        h4_chunks
            .iter()
            .map(|c| &c.heading_path)
            .collect::<Vec<_>>()
    );
}

#[test]
fn chunker_splits_oversized_code_without_function_boundaries() {
    // Codex review case: oversized code with NO function boundaries must still
    // be split so every sub-chunk is ≤ max_tokens (fixed-size windows were not
    // enough for minified/long-line content).
    let cfg = default_config();
    let mut code = String::from("```\n");
    // 500 identical short lines — no fn/def/class boundaries.
    for _ in 0..500 {
        code.push_str("let x = 42; // a line\n");
    }
    code.push_str("```\n");
    let md = format!("# Plain\n\n{code}\n");

    let chunks = chunk_markdown(&md, &cfg);
    let code_chunks: Vec<_> = chunks
        .iter()
        .filter(|c| c.chunk_type == ChunkType::Code)
        .collect();
    assert!(
        code_chunks.len() > 1,
        "oversized plain code must be split, got {}",
        code_chunks.len()
    );
    for c in &code_chunks {
        let n = count_tokens(&c.text);
        assert!(
            n <= cfg.max_tokens as usize,
            "plain-code sub-chunk {} exceeds max_tokens ({} > {})",
            c.idx,
            n,
            cfg.max_tokens
        );
    }
}

#[test]
fn chunker_hard_splits_single_oversized_line() {
    // Pathological case: one extremely long line inside a code block.
    let cfg = default_config();
    let mut code = String::from("```\n");
    // A single line with many tokens, well over max.
    code.push_str(&"x ".repeat(2000));
    code.push_str("\n```\n");
    let md = format!("# Long\n\n{code}\n");

    let chunks = chunk_markdown(&md, &cfg);
    let code_chunks: Vec<_> = chunks
        .iter()
        .filter(|c| c.chunk_type == ChunkType::Code)
        .collect();
    assert!(
        !code_chunks.is_empty(),
        "long-line code must still produce chunks"
    );
    for c in &code_chunks {
        assert!(
            count_tokens(&c.text) <= cfg.max_tokens as usize,
            "long-line sub-chunk {} exceeds max_tokens",
            c.idx
        );
    }
}
