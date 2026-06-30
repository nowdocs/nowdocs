//! Ingest pipeline: markdown directory -> chunks -> embeddings -> store + manifest.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::cache;
use crate::chunker::{self, Chunk};
use crate::embedder;
use crate::input;
use crate::manifest::{self, EmbedderSpec, LegalSpec, Manifest, RefreshSpec, RetrievalSpec, SourceSpec};
use crate::store::Store;

pub struct IngestStats {
    pub files: u32,
    pub chunks: u32,
}

/// Ingest a directory of markdown files into a docset store.
pub fn ingest_dir(dir: &Path, docset: &str) -> Result<IngestStats> {
    let docset = input::validate_docset(docset)?;
    cache::ensure_layout()?;

    // Collect chunks across all md files.
    let mut chunks: Vec<Chunk> = Vec::new();
    let mut files: u32 = 0;
    for entry in walk_md(dir)? {
        files += 1;
        let md = std::fs::read_to_string(&entry)
            .with_context(|| format!("read {}", entry.display()))?;
        let mut file_chunks = chunker::chunk_markdown(&md, &chunker::default_config());
        let rel = entry.strip_prefix(dir).unwrap_or(&entry).to_string_lossy().to_string();
        for c in &mut file_chunks {
            c.source_url = rel.clone();
        }
        chunks.extend(file_chunks);
    }

    // Reassign global idx (chunker's idx is per-file from 0).
    for (i, c) in chunks.iter_mut().enumerate() {
        c.idx = i as u32;
    }

    // Embed + insert. Empty dir skips embedder load but still opens (empty) store.
    if !chunks.is_empty() {
        let emb = embedder::Embedder::load()?;
        let vectors: Vec<Vec<f32>> = chunks
            .iter()
            .map(|c| emb.embed(&c.text))
            .collect::<Result<_>>()?;
        let store = Store::open(&docset)?;
        store.insert(&chunks, &vectors)?;
    } else {
        let _ = Store::open(&docset)?;
    }

    // Build + validate + write manifest.
    let manifest = build_manifest(&docset, chunks.len() as u32);
    manifest::validate(&manifest)?;
    std::fs::write(cache::manifest_path(&docset), serde_json::to_string_pretty(&manifest)?)?;

    Ok(IngestStats { files, chunks: chunks.len() as u32 })
}

/// Recursively collect `*.md` paths under `dir`, sorted for determinism.
fn walk_md(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d)? {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().map_or(false, |e| e == "md") {
                out.push(path);
            }
        }
    }
    out.sort();
    Ok(out)
}

/// Build the v1 manifest with locked embedder provenance.
fn build_manifest(docset: &str, chunk_count: u32) -> Manifest {
    let (model_id, model_revision, model_sha256) = embedder::provenance();
    Manifest {
        docset: docset.to_string(),
        doc_version: "1.0.0".to_string(),
        nowdocs_schema_version: 1,
        embedder: EmbedderSpec {
            model_id: model_id.to_string(),
            model_version: "jina-embeddings-v2-small-en".to_string(),
            model_revision: model_revision.to_string(),
            model_sha256: model_sha256.to_string(),
            vector_dim: 512,
            engine: "candle".to_string(),
            dtype: "f16".to_string(),
        },
        retrieval: RetrievalSpec {
            tokenizer: "default".to_string(),
            chunk_size_tokens: 512,
            window_tokens: 2048,
        },
        source: SourceSpec {
            entry_url: String::new(),
            source_url: String::new(),
            scraped_at: String::new(),
            chunk_count,
        },
        legal: LegalSpec {
            license: "MIT".to_string(),
            copyright_holder: String::new(),
            attribution: String::new(),
        },
        refresh_strategy: RefreshSpec {
            tier: "community".to_string(),
            auto_days: 0,
        },
    }
}
