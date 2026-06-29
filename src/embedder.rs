use anyhow::{Context, Result};
use candle_core::{DType, Device, Module, Tensor};
use candle_transformers::models::jina_bert::{BertModel, Config, PositionEmbeddingType};
use hf_hub::api::sync::Api;
use tokenizers::Tokenizer;

const MODEL_ID: &str = "jinaai/jina-embeddings-v2-small-en";
const VECTOR_DIM: usize = 512;

// TODO(S0-spike): pin a real revision commit SHA + sha256 before Wave 2 (A3 integrity).
// For the spike, hf-hub default (latest main) is acceptable to validate the path.
// If available, record the resolved revision here for Task 2a to harden.

pub struct Embedder {
    model: BertModel,
    tokenizer: Tokenizer,
}

impl Embedder {
    pub fn load() -> Result<Self> {
        let api = Api::new().context("hf-hub api")?;
        let repo = api.model(MODEL_ID.to_string());

        // Fetch files. Prefer safetensors; fall back to pytorch pickle if necessary.
        let weights = repo
            .get("model.safetensors")
            .or_else(|_| repo.get("pytorch_model.bin"))
            .context("fetch model weights (model.safetensors or pytorch_model.bin)")?;

        let config_path = repo.get("config.json").context("fetch config.json")?;
        sanitize_config(&config_path).context("sanitize config.json (remove auto_map)")?;

        let tok_path = repo.get("tokenizer.json").context("fetch tokenizer.json")?;

        // candle-transformers 0.11 exposes jina_bert::BertModel (not JinaBertModel).
        // v2-small config: hidden_size=512, layers=4, heads=8, intermediate=2048, alibi.
        let config = Config::new(
            30528,                        // vocab_size
            VECTOR_DIM,                   // hidden_size
            4,                            // num_hidden_layers
            8,                            // num_attention_heads
            2048,                         // intermediate_size
            candle_nn::Activation::Gelu,  // hidden_act (geglu gate uses gelu)
            8192,                         // max_position_embeddings
            2,                            // type_vocab_size
            0.02,                         // initializer_range
            1e-12,                        // layer_norm_eps
            0,                            // pad_token_id
            PositionEmbeddingType::Alibi, // position_embedding_type
        );

        // candle-transformers 0.11's jina_bert builds the ALiBi bias in F32 and
        // broadcasts it into the hidden activations, so weights must load as F32
        // even though the safetensors file stores F16-compatible values. Loading
        // F16 triggers `dtype mismatch in add, lhs: F16, rhs: F32`.
        let vb = if weights.extension().map_or(false, |e| e == "safetensors") {
            // SAFETY: mmap of a read-only model file in the HF cache; the file is
            // not modified concurrently (we only read / sanitize config.json).
            unsafe {
                candle_nn::VarBuilder::from_mmaped_safetensors(&[weights], DType::F32, &Device::Cpu)
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
        // Mean-pool over the sequence dimension (dim 1), matching the Python
        // sentence-transformers path with normalize_embeddings=false.
        let pooled = out.mean(1).context("mean pool")?;
        let v = pooled
            .squeeze(0)
            .context("squeeze")?
            .to_vec1::<f32>()
            .context("to_vec1")?;
        Ok(v)
    }
}

/// Remove `auto_map` from the HF config.json to prevent arbitrary code execution
/// if the file is later consumed by a Python transformers pipeline (A3).
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
