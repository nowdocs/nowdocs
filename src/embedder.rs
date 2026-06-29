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

/// Spec for loading an embedder with pinned provenance.
pub struct EmbedderSpec {
    pub model_id: String,
    pub model_revision: String,
    pub model_sha256: String,
}

pub struct Embedder {
    model: BertModel,
    tokenizer: Tokenizer,
}

impl Embedder {
    /// Load the default embedder with pinned revision + sha256.
    pub fn load() -> Result<Self> {
        let spec = EmbedderSpec {
            model_id: DEFAULT_MODEL_ID.to_string(),
            model_revision: DEFAULT_REVISION.to_string(),
            model_sha256: DEFAULT_SHA256.to_string(),
        };
        Self::load_for(&spec)
    }

    /// Load an embedder for the given spec: pin revision, download, verify sha256,
    /// sanitize config, and build the candle model.
    pub fn load_for(spec: &EmbedderSpec) -> Result<Self> {
        // Route hf-hub cache under nowdocs cache layout.
        // Set HF_HOME so hf-hub uses ~/.cache/nowdocs/models/<model_id>/
        // as its base (creating hub/ inside it).
        let model_cache = crate::cache::model_path(&spec.model_id);
        std::fs::create_dir_all(&model_cache).context("create model cache dir")?;
        // SAFETY: single-threaded embedder load; no concurrent HF_HOME readers.
        unsafe { std::env::set_var("HF_HOME", &model_cache) };

        let api = ApiBuilder::from_env()
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

        let vb = if weights.extension().map_or(false, |e| e == "safetensors") {
            // SAFETY: mmap of a read-only model file in the HF cache.
            unsafe {
                candle_nn::VarBuilder::from_mmaped_safetensors(
                    &[weights.clone()],
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

        let tokenizer = Tokenizer::from_file(tok_path)
            .map_err(|e| anyhow::anyhow!("tokenizer: {e}"))?;

        Ok(Self { model, tokenizer })
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let enc = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| anyhow::anyhow!("encode: {e}"))?;
        let ids = enc.get_ids();
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

/// Remove `auto_map` from the HF config.json to prevent arbitrary code execution (A3).
fn sanitize_config(path: &std::path::Path) -> Result<()> {
    let text = std::fs::read_to_string(path).context("read config.json")?;
    let mut val: serde_json::Value = serde_json::from_str(&text).context("parse config.json")?;
    if let Some(obj) = val.as_object_mut() {
        obj.remove("auto_map");
    }
    std::fs::write(path, serde_json::to_string_pretty(&val).context("serialize config")?)
        .context("write sanitized config.json")?;
    Ok(())
}

/// Compute SHA256 hex digest of a file.
fn sha256_hex(path: &std::path::Path) -> Result<String> {
    let bytes = std::fs::read(path).context("read file for sha256")?;
    let hash = Sha256::digest(&bytes);
    Ok(format!("{hash:x}"))
}
