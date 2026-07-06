//! Smoke-test: real retrieval to verify a docset installation works end-to-end.

use std::time::Instant;

use anyhow::{bail, Context, Result};
use serde::Serialize;

use crate::retrieve;

const DEFAULT_QUERY: &str = "installation configuration example";
const DEFAULT_TOP_K: u32 = 3;

/// Result of a smoke test.
#[derive(Debug, Serialize)]
pub struct SmokeResult {
    pub docset: String,
    pub query: String,
    pub elapsed_ms: u64,
    pub result_count: usize,
    pub results: Vec<SmokeHit>,
}

#[derive(Debug, Serialize)]
pub struct SmokeHit {
    pub rank: usize,
    pub score: f32,
    pub heading: String,
    pub source_url: String,
    pub chunk_idx: u32,
    pub preview: String,
}

/// Run a smoke test: real retrieval on an installed docset.
pub fn smoke(docset: &str, query: Option<&str>, top_k: Option<u32>) -> Result<SmokeResult> {
    let query = query.unwrap_or(DEFAULT_QUERY);
    let top_k = top_k.unwrap_or(DEFAULT_TOP_K);

    // Verify docset is installed (manifest exists).
    let manifest_path = crate::cache::manifest_path(docset);
    if !manifest_path.is_file() {
        bail!(
            "docset {docset:?} not found — run `nowdocs install {docset}` or `nowdocs ingest` first"
        );
    }

    let start = Instant::now();

    // Run real retrieval (embed + hybrid search).
    let search_result = retrieve::search(docset, query, None, Some(top_k))
        .context("retrieval failed — model may need downloading; try `nowdocs doctor --model`")?;

    let elapsed_ms = start.elapsed().as_millis() as u64;

    // retrieve::search returns hit-first window (hits + neighbor context).
    // Limit to the requested top_k hits — neighbors are context, not results.
    let results: Vec<SmokeHit> = search_result
        .chunks
        .into_iter()
        .take(top_k as usize)
        .enumerate()
        .map(|(i, c)| {
            let preview = truncate_text(&c.text, 120);
            SmokeHit {
                rank: i + 1,
                score: c.score.unwrap_or(0.0),
                heading: c.heading_path,
                source_url: c.source_url,
                chunk_idx: c.chunk_idx,
                preview,
            }
        })
        .collect();

    let result_count = results.len();

    Ok(SmokeResult {
        docset: docset.to_string(),
        query: query.to_string(),
        elapsed_ms,
        result_count,
        results,
    })
}

fn truncate_text(text: &str, max_len: usize) -> String {
    let text = text.trim();
    if text.len() <= max_len {
        text.to_string()
    } else {
        // Find a char boundary at or before max_len to avoid panic on multi-byte UTF-8.
        let mut end = max_len;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}...", &text[..end])
    }
}

/// Format smoke result for human output.
pub fn format_human(result: &SmokeResult) -> String {
    let mut out = String::new();
    out.push_str(&format!("smoke ok: {}\n", result.docset));
    out.push_str(&format!("query: {}\n", result.query));
    out.push_str(&format!(
        "results: {} in {}ms\n",
        result.result_count, result.elapsed_ms
    ));
    for hit in &result.results {
        out.push_str(&format!(
            "{}. heading=\"{}\" source={} chunk={}\n   {}\n",
            hit.rank, hit.heading, hit.source_url, hit.chunk_idx, hit.preview
        ));
    }
    out
}

/// Format smoke result as JSON.
pub fn format_json(result: &SmokeResult) -> Result<String> {
    serde_json::to_string_pretty(result).context("failed to serialize smoke result")
}
