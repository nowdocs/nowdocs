use serde::{Deserialize, Serialize};

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

pub fn parse_manifest(_json: &str) -> anyhow::Result<Manifest> {
    todo!("1b")
}
pub fn validate(_m: &Manifest) -> anyhow::Result<()> {
    todo!("1b")
}
