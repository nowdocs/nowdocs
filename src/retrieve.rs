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
    // Over-fetch then de-dup by source_url: hybrid often returns multiple
    // chunks from the same hub file (installation.md, glossary.md,
    // backend-for-frontend.md) that match moderately on both channels — RRF
    // fusion floats them into the top-K and squeezes the specific expected
    // chunk to rank 9+ (see eval `test_eval_nextjs_diagnose`). Keeping only the
    // highest-scoring chunk per source_url lets diverse files reach the top-K.
    // The window pass below re-attaches same-file neighbor chunks as context,
    // so per-file coverage is preserved.
    let raw_k = (top_k * 3).max(15) as usize;
    let hits = store.hybrid_search(&query_vector, &query, raw_k)?;
    let hits = dedup_by_source_url(hits);
    let hits: Vec<SearchHit> = hits.into_iter().take(top_k as usize).collect();

    if hits.is_empty() {
        return Ok(SearchResult {
            chunks: vec![],
            tokens_returned: 0,
            truncated: false,
        });
    }

    // Build the neighbor window hit-first: all hybrid hits lead (rank 1..N),
    // then their `[hit-1, hit+1]` neighbors follow as context. This keeps a
    // hit recalled by hybrid inside the top-K — the earlier per-hit interleave
    // (`[hit, hit-1, hit+1]`) let hit1..4's neighbors push hit5 from hybrid
    // rank 5 to retrieve rank ~7, squeezing it out of recall@5. See
    // `window_ids_for`.
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

/// Keep only the highest-scoring hit per `source_url`. `hits` must be in
/// score-descending order (as returned by `Store::hybrid_search`), so the first
/// occurrence of each source_url is the best — later duplicates from the same
/// file are dropped. See `search` for why hub-file dedup lifts recall.
fn dedup_by_source_url(hits: Vec<SearchHit>) -> Vec<SearchHit> {
    let mut seen = std::collections::HashSet::new();
    hits.into_iter()
        .filter(|h| seen.insert(h.source_url.clone()))
        .collect()
}

/// Build the relevance-ordered neighbor-window index list from hybrid search
/// hits. `hits` must be in score-descending order (as returned by
/// `Store::hybrid_search`).
///
/// Two passes keep every hybrid hit inside the top-K: (1) all hits in
/// relevance order, then (2) each hit's `[hit-1, hit+1]` neighbors as context.
/// The earlier per-hit interleave (`[hit, hit-1, hit+1]`) let hit1..4's
/// neighbors push hit5 from hybrid rank 5 to retrieve rank ~7 — a hit recalled
/// by hybrid was squeezed out of the top-K window by *other hits'* context.
/// Hit-first ordering keeps the window additive: it only ever appends context
/// below the hits, never displaces them. (Note: `search` de-dups hits by
/// source_url before this, so the hits here are already one-per-file; the
/// window then re-expands per-file coverage via neighbors.) Indices are
/// clamped to `[0, chunk_count)`.
pub fn window_ids_for(hits: &[SearchHit], chunk_count: u32) -> Vec<u32> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Pass 1: hits in relevance order so rank 1..N lead the result.
    for hit in hits {
        let idx = hit.chunk_idx;
        if idx < chunk_count && seen.insert(idx) {
            out.push(idx);
        }
    }

    // Pass 2: neighbor context [hit-1, hit+1] per hit, relevance order.
    for hit in hits {
        for delta in [-1i32, 1] {
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
