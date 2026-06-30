//! Retrieval pipeline: embed query -> hybrid search -> neighbor-window assembly.

use anyhow::{Context, Result};

use crate::chunker::ChunkType;
use crate::embedder::{self, EmbedderSpec};
use crate::input::{resolve_max_tokens, resolve_top_k, validate_docset, validate_query};
use crate::manifest;
use crate::store::Store;
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

    // Collect neighbor window indices: for each hit, take chunk_idx-1, chunk_idx, chunk_idx+1.
    let chunk_count = manifest.source.chunk_count as u32;
    let mut window_ids: Vec<u32> = Vec::new();
    for hit in &hits {
        for delta in [-1i32, 0, 1] {
            let idx = hit.chunk_idx as i32 + delta;
            if idx >= 0 && (idx as u32) < chunk_count {
                window_ids.push(idx as u32);
            }
        }
    }
    window_ids.sort_unstable();
    window_ids.dedup();

    // Fetch window chunks from lancedb by chunk_idx.
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

    // Assemble within max_tokens budget.
    assemble_result(window_chunks, max_tokens)
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
