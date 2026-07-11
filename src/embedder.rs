use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use anyhow::{Context, Result};
use candle_core::{DType, Device, Module, Tensor};
use candle_transformers::models::jina_bert::{BertModel, Config, PositionEmbeddingType};
use hf_hub::api::sync::ApiBuilder;
use hf_hub::{Repo, RepoType};
use sha2::{Digest, Sha256};
use tokenizers::Tokenizer;

/// Pinned provenance for the default embedder model (S0 spike results).
const DEFAULT_MODEL_ID: &str = "jinaai/jina-embeddings-v2-small-en";
const DEFAULT_REVISION: &str = "44e7d1d6caec8c883c2d4b207588504d519788d0";
const DEFAULT_SHA256: &str = "c9a9a7ec012d01efd780474fbb65e25917f3a2aebdff84b5f87daa00f7e90b27";
const VECTOR_DIM: usize = 512;

/// jina-v2-small `max_position_embeddings`. Inputs longer than this are
/// truncated (to `MAX_POSITION_TOKENS - 1`) before the forward pass rather than
/// panicking inside candle (N7 token guard).
pub(crate) const MAX_POSITION_TOKENS: usize = 8192;

/// Pinned provenance for manifest generation (model_id, revision, sha256).
pub fn provenance() -> (&'static str, &'static str, &'static str) {
    (DEFAULT_MODEL_ID, DEFAULT_REVISION, DEFAULT_SHA256)
}

/// Spec for loading an embedder with pinned provenance.
///
/// `Hash`/`Eq` so it can key the process-level embedder cache (N3).
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct EmbedderSpec {
    pub model_id: String,
    pub model_revision: String,
    pub model_sha256: String,
}

pub struct Embedder {
    model: BertModel,
    tokenizer: Tokenizer,
}

/// Process-level embedder cache (N3): a model (~66 MB weights + tokenizer +
/// tokio runtime) is loaded at most once per `EmbedderSpec` and then shared via
/// `Arc`. The MCP `serve` loop warms the default entry at startup; CLI commands
/// populate it lazily on the first search. Failed loads are never inserted, so a
/// transient error (e.g. sha256 mismatch) does not poison the cache.
static EMBEDDER_CACHE: OnceLock<Mutex<HashMap<EmbedderSpec, Arc<Embedder>>>> = OnceLock::new();

fn embedder_cache() -> &'static Mutex<HashMap<EmbedderSpec, Arc<Embedder>>> {
    EMBEDDER_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Cap a token-id slice to the model's max position (`MAX_POSITION_TOKENS - 1`),
/// logging a warning when truncation happens. Pure so the cap logic (the only
/// part that could panic on an oversized input) can be unit-tested without
/// running a full 8191-token forward pass (N7).
pub(crate) fn cap_to_max_position(ids: &[u32]) -> &[u32] {
    if ids.len() > MAX_POSITION_TOKENS {
        let max = MAX_POSITION_TOKENS - 1;
        eprintln!(
            "chunk truncated from {} to {max} tokens for embedding",
            ids.len()
        );
        &ids[..max]
    } else {
        ids
    }
}

impl Embedder {
    /// Load the default embedder with pinned revision + sha256, returning a
    /// process-shared `Arc` (N3 cache).
    pub fn load() -> Result<Arc<Embedder>> {
        let spec = EmbedderSpec {
            model_id: DEFAULT_MODEL_ID.to_string(),
            model_revision: DEFAULT_REVISION.to_string(),
            model_sha256: DEFAULT_SHA256.to_string(),
        };
        Self::load_for(&spec)
    }

    /// Load an embedder for the given spec, served from the process cache when
    /// warm. On a miss this pins the revision, downloads, verifies sha256,
    /// sanitizes the config, builds the candle model, and inserts it into the
    /// cache. Failed loads are not cached, so a transient error (e.g. a sha256
    /// mismatch) does not poison subsequent loads.
    pub fn load_for(spec: &EmbedderSpec) -> Result<Arc<Embedder>> {
        if let Some(hit) = embedder_cache().lock().unwrap().get(spec).cloned() {
            return Ok(hit);
        }
        let loaded = Self::load_uncached(spec)?;
        let arc = Arc::new(loaded);
        embedder_cache()
            .lock()
            .unwrap()
            .insert(spec.clone(), Arc::clone(&arc));
        Ok(arc)
    }

    fn load_uncached(spec: &EmbedderSpec) -> Result<Embedder> {
        // Route the hf-hub cache under the nowdocs cache layout by passing the
        // cache dir straight to the builder (M13). This avoids mutating any
        // process-wide downloader env var (load runs inside a tokio runtime) and
        // keeps the door open for a second model with a distinct cache dir.
        let model_cache = crate::cache::model_path(&spec.model_id);
        std::fs::create_dir_all(&model_cache).context("create model cache dir")?;

        let api = ApiBuilder::new()
            .with_cache_dir(model_cache)
            .with_progress(false)
            .build()
            .context("hf-hub api")?;

        let repo = api.repo(Repo::with_revision(
            spec.model_id.clone(),
            RepoType::Model,
            spec.model_revision.clone(),
        ));

        // Fetch files.
        let weights = repo
            .get("model.safetensors")
            .or_else(|_| repo.get("pytorch_model.bin"))
            .context("fetch model weights")?;

        let config_path = repo.get("config.json").context("fetch config.json")?;
        sanitize_config(&config_path).context("sanitize config.json (remove auto_map)")?;

        let tok_path = repo.get("tokenizer.json").context("fetch tokenizer.json")?;

        // Verify sha256 of model weights (A3 integrity).
        let actual_sha = sha256_hex(&weights)?;
        if actual_sha != spec.model_sha256 {
            // Delete the actual blob file (weights is a symlink into snapshots/).
            let real_path = std::fs::canonicalize(&weights).unwrap_or(weights.clone());
            let _ = std::fs::remove_file(&real_path);
            let _ = std::fs::remove_file(&weights);
            anyhow::bail!(
                "model integrity check failed: expected sha256={}, got={actual_sha}. File deleted.",
                spec.model_sha256
            );
        }

        // Build candle model. Weights load as F32 (candle 0.11 ALiBi bias is hardcoded F32).
        let config = Config::new(
            30528,
            VECTOR_DIM,
            4,
            8,
            2048,
            candle_nn::Activation::Gelu,
            8192,
            2,
            0.02,
            1e-12,
            0,
            PositionEmbeddingType::Alibi,
        );

        let vb = if weights.extension().is_some_and(|e| e == "safetensors") {
            // SAFETY: mmap of a read-only model file in the HF cache.
            unsafe {
                candle_nn::VarBuilder::from_mmaped_safetensors(
                    std::slice::from_ref(&weights),
                    DType::F32,
                    &Device::Cpu,
                )
                .context("mmap safetensors")?
            }
        } else {
            candle_nn::VarBuilder::from_pth(&weights, DType::F32, &Device::Cpu)
                .context("load pytorch_model.bin")?
        };

        let model = BertModel::new(vb, &config).context("load jina-bert")?;

        let tokenizer =
            Tokenizer::from_file(tok_path).map_err(|e| anyhow::anyhow!("tokenizer: {e}"))?;

        Ok(Self { model, tokenizer })
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let enc = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| anyhow::anyhow!("encode: {e}"))?;
        let all_ids = enc.get_ids();
        // N7 token guard: never feed more than the model's max position. Truncate
        // to `max - 1` and warn instead of letting candle panic on an oversized
        // input (the embedder is the last line of defense behind the chunker's
        // oversized-block split).
        let ids = cap_to_max_position(all_ids);
        let input = Tensor::new(ids, &Device::Cpu)?
            .reshape((1, ids.len()))?
            .to_dtype(DType::I64)?;

        let out = self.model.forward(&input).context("forward")?;
        let pooled = out.mean(1).context("mean pool")?;
        let v = pooled
            .squeeze(0)
            .context("squeeze")?
            .to_vec1::<f32>()
            .context("to_vec1")?;
        Ok(v)
    }
}

/// Best-effort warmup of the default embedder for the MCP `serve` loop (N3).
///
/// To keep `nowdocs serve` hermetic and offline-safe — and to avoid a surprise
/// ~66 MB download on every startup — this only loads when the default model is
/// already materialized in the local cache (a previous `search`/`ingest` warmed
/// it). On a cold cache it returns immediately and the first search loads on
/// demand. Load errors are logged, never fatal: a broken cache must not prevent
/// the server from answering `initialize`/`tools/list`.
pub fn preload_default_embedder() {
    if !default_model_cached() {
        return;
    }
    if let Err(e) = Embedder::load() {
        eprintln!("nowdocs: embedder preload failed (will retry on first search): {e}");
    }
}

/// True when the default model's weights are already present in the nowdocs
/// model cache (hf-hub `models--<id>/snapshots/<rev>/` layout). Used to decide
/// whether `serve`-time preloading is free (warm) or would require a download
/// (cold — skip and load lazily instead).
pub fn default_model_cached() -> bool {
    default_weights_path().is_some()
}

fn default_weights_path() -> Option<std::path::PathBuf> {
    let cache = crate::cache::model_path(DEFAULT_MODEL_ID);
    let repo_dir = cache.join(format!("models--{}", DEFAULT_MODEL_ID.replace('/', "--")));
    let snapshots = repo_dir.join("snapshots").join(DEFAULT_REVISION);
    for name in ["model.safetensors", "pytorch_model.bin"] {
        let p = snapshots.join(name);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// Remove `auto_map` from the HF config.json to prevent arbitrary code execution (A3).
fn sanitize_config(path: &std::path::Path) -> Result<()> {
    let text = std::fs::read_to_string(path).context("read config.json")?;
    let mut val: serde_json::Value = serde_json::from_str(&text).context("parse config.json")?;
    if let Some(obj) = val.as_object_mut() {
        obj.remove("auto_map");
    }
    std::fs::write(
        path,
        serde_json::to_string_pretty(&val).context("serialize config")?,
    )
    .context("write sanitized config.json")?;
    Ok(())
}

/// Compute SHA256 hex digest of a file.
fn sha256_hex(path: &std::path::Path) -> Result<String> {
    let bytes = std::fs::read(path).context("read file for sha256")?;
    let hash = Sha256::digest(&bytes);
    Ok(hash
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cap_to_max_position_truncates_oversized_without_panic() {
        // Far beyond 8192 tokens: must cap to max-1 and never panic on the slice.
        let big: Vec<u32> = (0..20_000).collect();
        let capped = cap_to_max_position(&big);
        assert_eq!(capped.len(), MAX_POSITION_TOKENS - 1);
        assert_eq!(capped[0], 0);
        assert_eq!(*capped.last().unwrap(), (MAX_POSITION_TOKENS - 2) as u32);
    }

    #[test]
    fn cap_to_max_position_leaves_short_inputs_intact() {
        let small: Vec<u32> = (0..100).collect();
        let capped = cap_to_max_position(&small);
        assert_eq!(capped.len(), 100);
        assert!(std::ptr::eq(capped.as_ptr(), small.as_ptr()));
    }
}
