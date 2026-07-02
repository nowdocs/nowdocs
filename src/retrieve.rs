//! Retrieval pipeline: embed query -> hybrid search -> neighbor-window assembly.

use anyhow::{Context, Result};

use crate::chunker::ChunkType;
use crate::embedder::{self, EmbedderSpec};
use crate::input::{resolve_max_tokens, resolve_top_k, validate_docset, validate_query};
use crate::manifest;
use crate::store::{SearchHit, Store};
use crate::token::count_tokens;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub chunks: Vec<ResultChunk>,
    pub tokens_returned: u32,
    pub truncated: bool,
}

#[derive(Debug, Clone)]
pub struct ResultChunk {
    pub chunk_idx: u32,
    pub heading_path: String,
    pub source_url: String,
    pub api_version: Option<String>,
    pub chunk_type: ChunkType,
    pub text: String,
}

pub fn search(
    docset: &str,
    query: &str,
    max_tokens: Option<u32>,
    top_k: Option<u32>,
) -> Result<SearchResult> {
    let docset = validate_docset(docset)?;
    let query = validate_query(query)?;
    let max_tokens = resolve_max_tokens(max_tokens);
    let top_k = resolve_top_k(top_k);

    // Load and validate manifest.
    let manifest_path = crate::cache::manifest_path(&docset);
    let manifest_json = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read manifest for docset {docset:?}"))?;
    let manifest: manifest::Manifest = manifest::parse_manifest(&manifest_json)?;
    manifest::validate(&manifest)?;

    // Build embedder spec from manifest (model-version lock).
    let embedder_spec = EmbedderSpec {
        model_id: manifest.embedder.model_id.clone(),
        model_revision: manifest.embedder.model_revision.clone(),
        model_sha256: manifest.embedder.model_sha256.clone(),
    };
    let embedder = embedder::Embedder::load_for(&embedder_spec)?;

    // Embed query and run hybrid search.
    let query_vector = embedder.embed(&query)?;
    let store = Store::open(&docset)?;
    let hits = store.hybrid_search(&query_vector, &query, top_k as usize)?;

    if hits.is_empty() {
        return Ok(SearchResult {
            chunks: vec![],
            tokens_returned: 0,
            truncated: false,
        });
    }

    // Build the neighbor window in relevance-first order: for each hit
    // (already ranked by score desc from `hybrid_search`), emit
    // `[hit, hit-1, hit+1]` so the hit itself leads and its adjacent chunks
    // follow as context. Dedup keeps first-seen position, so a higher-ranked
    // hit's chunk precedes a lower-ranked hit's context. This fixes bug#1: the
    // old code sorted window_ids by chunk_idx, discarding relevance and making
    // output order independent of the query (MRR 0.65 was pure idx luck).
    let chunk_count = manifest.source.chunk_count as u32;
    let window_ids = window_ids_for(&hits, chunk_count);

    // Fetch window chunks by chunk_idx (store returns them idx-ascending), then
    // restore the relevance-first window order.
    let window_hits = store.fetch_by_idx(&window_ids)?;
    let window_chunks: Vec<ResultChunk> = window_hits
        .into_iter()
        .map(|h| ResultChunk {
            chunk_idx: h.chunk_idx,
            heading_path: h.heading_path,
            source_url: h.source_url,
            api_version: h.api_version,
            chunk_type: h.chunk_type,
            text: h.text,
        })
        .collect();
    let window_chunks = reorder_to_window(window_chunks, &window_ids);

    // Assemble within max_tokens budget.
    assemble_result(window_chunks, max_tokens)
}

/// Build the relevance-ordered neighbor-window index list from hybrid search
/// hits. `hits` must be in score-descending order (as returned by
/// `Store::hybrid_search`). For each hit, emits `[hit, hit-1, hit+1]` so the
/// hit itself leads and its adjacent chunks follow as context. Deduplication
/// keeps the first-seen position, so a higher-ranked hit's chunk always
/// precedes a lower-ranked hit's context. Indices are clamped to
/// `[0, chunk_count)`.
pub fn window_ids_for(hits: &[SearchHit], chunk_count: u32) -> Vec<u32> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for hit in hits {
        for delta in [0i32, -1, 1] {
            let idx = hit.chunk_idx as i32 + delta;
            if idx >= 0 && (idx as u32) < chunk_count {
                let u = idx as u32;
                if seen.insert(u) {
                    out.push(u);
                }
            }
        }
    }
    out
}

/// Reorder chunks fetched by idx (ascending) back into the relevance-first
/// window order defined by `window_ids`. Chunks whose idx is absent from
/// `window_ids` sort to the end (defensive — `fetch_by_idx` only returns
/// requested ids in practice).
pub fn reorder_to_window(chunks: Vec<ResultChunk>, window_ids: &[u32]) -> Vec<ResultChunk> {
    let mut order: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for (i, &id) in window_ids.iter().enumerate() {
        order.insert(id, i);
    }
    let mut chunks = chunks;
    chunks.sort_by_key(|c| order.get(&c.chunk_idx).copied().unwrap_or(usize::MAX));
    chunks
}

fn assemble_result(chunks: Vec<ResultChunk>, max_tokens: u32) -> Result<SearchResult> {
    let mut selected = Vec::new();
    let mut tokens_used: u32 = 0;
    let mut truncated = false;

    for chunk in chunks {
        let n = count_tokens(&chunk.text) as u32;
        if selected.is_empty() {
            selected.push(chunk);
            tokens_used += n;
            continue;
        }
        if tokens_used + n > max_tokens {
            truncated = true;
            break;
        }
        selected.push(chunk);
        tokens_used += n;
    }

    // If even the first chunk exceeds budget, still return it but mark truncated.
    if selected.len() == 1 && tokens_used > max_tokens {
        truncated = true;
    }

    Ok(SearchResult {
        chunks: selected,
        tokens_returned: tokens_used,
        truncated,
    })
}
