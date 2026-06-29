#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkType {
    Code,
    Info,
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub idx: u32,
    pub heading_path: String,
    pub source_url: String,
    pub api_version: Option<String>,
    pub chunk_type: ChunkType,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct ChunkConfig {
    pub min_tokens: u32,
    pub max_tokens: u32,
    pub target_tokens: u32,
    pub window_tokens: u32,
}

pub fn default_config() -> ChunkConfig {
    ChunkConfig {
        min_tokens: 256,
        max_tokens: 512,
        target_tokens: 384,
        window_tokens: 2048,
    }
}

pub fn chunk_markdown(_md: &str, _cfg: &ChunkConfig) -> Vec<Chunk> {
    todo!("1c")
}
