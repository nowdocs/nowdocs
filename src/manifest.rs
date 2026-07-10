use serde::{Deserialize, Serialize};

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaCompatibility {
    Current,
    Older { found: u32, current: u32 },
    Newer { found: u32, current: u32 },
}

pub fn schema_compatibility(version: u32) -> SchemaCompatibility {
    match version.cmp(&CURRENT_SCHEMA_VERSION) {
        std::cmp::Ordering::Equal => SchemaCompatibility::Current,
        std::cmp::Ordering::Less => SchemaCompatibility::Older {
            found: version,
            current: CURRENT_SCHEMA_VERSION,
        },
        std::cmp::Ordering::Greater => SchemaCompatibility::Newer {
            found: version,
            current: CURRENT_SCHEMA_VERSION,
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub docset: String,
    pub doc_version: String,
    pub nowdocs_schema_version: u32,
    pub embedder: EmbedderSpec,
    pub retrieval: RetrievalSpec,
    pub source: SourceSpec,
    pub legal: LegalSpec,
    pub refresh_strategy: RefreshSpec,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedderSpec {
    pub model_id: String,
    pub model_version: String,
    pub model_revision: String,
    pub model_sha256: String,
    pub vector_dim: u32,
    pub engine: String,
    pub dtype: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalSpec {
    pub tokenizer: String,
    pub chunk_size_tokens: u32,
    pub window_tokens: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSpec {
    pub entry_url: String,
    pub source_url: String,
    pub scraped_at: String,
    pub chunk_count: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalSpec {
    pub license: String,
    pub copyright_holder: String,
    pub attribution: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshSpec {
    pub tier: String,
    pub auto_days: u32,
}

pub fn parse_manifest(json: &str) -> anyhow::Result<Manifest> {
    let m: Manifest = serde_json::from_str(json)?;
    Ok(m)
}

/// Validate install-context business invariants that `validate` does not cover.
///
/// `validate` checks the self-contained schema/model/license surface. This
/// checks the invariants that only make sense relative to a specific install:
/// the manifest's `docset` must equal the requested install name (identity
/// binding, complements Phase 0 S7), the docset must contain at least one
/// chunk, and it must carry at least one traceable upstream URL. License and
/// attribution are intentionally NOT re-checked here — `validate` already
/// enforces the allowlist and CC-BY-4.0 attribution. Callers should run
/// `validate` first, then this.
pub fn validate_manifest_for_docset(m: &Manifest, expected_docset: &str) -> anyhow::Result<()> {
    if m.docset != expected_docset {
        anyhow::bail!(
            "manifest docset {:?} does not match install name {:?}",
            m.docset,
            expected_docset
        );
    }
    if m.source.chunk_count == 0 {
        anyhow::bail!(
            "manifest for {:?} has chunk_count 0; refusing to install an empty docset",
            m.docset
        );
    }
    if m.source.source_url.trim().is_empty() && m.source.entry_url.trim().is_empty() {
        anyhow::bail!(
            "manifest for {:?} must carry at least one of source.source_url / source.entry_url",
            m.docset
        );
    }
    Ok(())
}

/// Validate manifest against v1 invariants:
/// - schema version must be 1 (only v1 supported)
/// - embedder must be the locked model (jina-v2-small, 512-dim, candle, f16)
/// - retrieval.tokenizer must be "default" (lindera reserved for v2)
/// - license must be allowlisted; CC-BY-4.0 requires non-empty attribution
pub fn validate(m: &Manifest) -> anyhow::Result<()> {
    match schema_compatibility(m.nowdocs_schema_version) {
        SchemaCompatibility::Current => {}
        SchemaCompatibility::Older { found, current } => anyhow::bail!(
            "unsupported older nowdocs_schema_version: {found} (current {current}); run `nowdocs rebuild {}` to migrate the local cache",
            m.docset
        ),
        SchemaCompatibility::Newer { found, current } => anyhow::bail!(
            "manifest schema {found} is newer than this nowdocs binary supports ({current}); upgrade nowdocs, or run `nowdocs rebuild {}` after downgrading only if you accept rebuilding the local cache",
            m.docset
        ),
    }
    if m.embedder.model_id != "jinaai/jina-embeddings-v2-small-en" {
        anyhow::bail!("embedder.model_id mismatch: {}", m.embedder.model_id);
    }
    if m.embedder.vector_dim != 512 {
        anyhow::bail!(
            "embedder.vector_dim must be 512, got {}",
            m.embedder.vector_dim
        );
    }
    if m.embedder.engine != "candle" {
        anyhow::bail!(
            "embedder.engine must be \"candle\", got {}",
            m.embedder.engine
        );
    }
    if m.embedder.dtype != "f16" {
        anyhow::bail!("embedder.dtype must be \"f16\", got {}", m.embedder.dtype);
    }
    if m.embedder.model_revision.trim().is_empty() {
        anyhow::bail!("embedder.model_revision must not be empty");
    }
    if m.embedder.model_sha256.trim().is_empty() {
        anyhow::bail!("embedder.model_sha256 must not be empty");
    }
    if m.retrieval.tokenizer != "default" {
        anyhow::bail!(
            "retrieval.tokenizer must be \"default\" (v1), got {}",
            m.retrieval.tokenizer
        );
    }
    match m.legal.license.as_str() {
        "MIT" | "Apache-2.0" => {}
        "CC-BY-4.0" => {
            if m.legal.attribution.trim().is_empty() {
                anyhow::bail!("CC-BY-4.0 license requires non-empty attribution");
            }
        }
        other => anyhow::bail!("license not allowlisted: {}", other),
    }
    Ok(())
}
