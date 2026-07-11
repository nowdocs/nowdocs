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
    /// Total measured wall time of the smoke retrieval work (`embed_ms + search_ms`).
    pub elapsed_ms: u64,
    /// M23: time to embed the query (model load + one forward pass), in ms.
    pub embed_ms: u64,
    /// M23: time for the full retrieve pipeline (embed + hybrid + MMR + window +
    /// assembly), in ms. Includes its own embed, so `search_ms - embed_ms` is a
    /// rough store/MMR/window cost.
    pub search_ms: u64,
    /// M23: tokens returned within the max_tokens budget.
    pub tokens_returned: u32,
    /// M23: whether the result was truncated to fit the token budget.
    pub truncated: bool,
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

    // M22: verify the docset is usable via the unified state model. Smoke needs
    // both a manifest (embedder spec) and a store (to search), so any partial
    // state bails early with a targeted hint instead of a cryptic retrieve error.
    match crate::cache::check_docset_state(docset) {
        crate::cache::InstalledDocsetState::NotInstalled => {
            bail!(
                "docset {docset:?} not found — run `nowdocs install {docset}` or `nowdocs ingest` first"
            );
        }
        crate::cache::InstalledDocsetState::ManifestOnly => {
            bail!(
                "docset {docset:?} has a manifest but no store — run `nowdocs rebuild {docset}` or `nowdocs install {docset}`"
            );
        }
        crate::cache::InstalledDocsetState::StoreOnly => {
            bail!(
                "docset {docset:?} has a store but no manifest — run `nowdocs install {docset}` to reinstall"
            );
        }
        _ => {}
    }

    // M23: time a standalone query embed first (warms the model and measures
    // one forward pass), then time the full retrieve pipeline separately, so
    // "slow search" can be attributed to embedding vs. store/MMR/window. The
    // embedder is process-cached (N3), so retrieve's internal reload is a cheap
    // handle clone after this probe.
    let embed_start = Instant::now();
    let embedder = crate::embedder::Embedder::load().context(
        "embedder load failed — model may need downloading; try `nowdocs doctor --model`",
    )?;
    embedder
        .embed(query)
        .context("query embed failed — try `nowdocs doctor --model`")?;
    let embed_ms = embed_start.elapsed().as_millis() as u64;

    let search_start = Instant::now();

    // Run real retrieval (embed + hybrid search).
    let search_result = retrieve::search(docset, query, None, Some(top_k))
        .context("retrieval failed — model may need downloading; try `nowdocs doctor --model`")?;

    let search_ms = search_start.elapsed().as_millis() as u64;
    let elapsed_ms = embed_ms + search_ms;
    let tokens_returned = search_result.tokens_returned;
    let truncated = search_result.truncated;

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
        embed_ms,
        search_ms,
        tokens_returned,
        truncated,
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
    // M23: embed/search split lets users attribute slow searches.
    out.push_str(&format!(
        "timing: embed_ms={} search_ms={} total_ms={}\n",
        result.embed_ms, result.search_ms, result.elapsed_ms
    ));
    out.push_str(&format!(
        "results: {} tokens_returned={} truncated={}\n",
        result.result_count, result.tokens_returned, result.truncated
    ));
    for hit in &result.results {
        out.push_str(&format!(
            "{}. score={:.2} heading=\"{}\" source={} chunk={}\n   {}\n",
            hit.rank, hit.score, hit.heading, hit.source_url, hit.chunk_idx, hit.preview
        ));
    }
    out
}

/// Format smoke result as JSON.
pub fn format_json(result: &SmokeResult) -> Result<String> {
    serde_json::to_string_pretty(result).context("failed to serialize smoke result")
}
