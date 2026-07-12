use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::chunker::{Chunk, ChunkType};
use crate::embedder::Embedder;
use crate::input;
use crate::manifest;
use crate::store::Store;

pub struct RegistryReleaseBuild {
    pub bundle_dir: PathBuf,
    pub output_dir: PathBuf,
    pub public_docset: String,
    pub source_url_base: String,
}

#[derive(Debug, Deserialize)]
struct ShareChunk {
    idx: u32,
    heading_path: String,
    source_url: String,
    #[serde(default)]
    api_version: Option<String>,
    chunk_type: String,
    text: String,
}

#[derive(Debug, Serialize)]
struct OutputChunk<'a> {
    idx: u32,
    heading_path: &'a str,
    source_url: &'a str,
    api_version: Option<&'a str>,
    chunk_type: &'a str,
    text: &'a str,
}

pub fn build_registry_release(input: &RegistryReleaseBuild) -> Result<PathBuf> {
    let public_docset = validate_public_docset(&input.public_docset)?;
    if !input.source_url_base.starts_with("https://") {
        anyhow::bail!("source URL base must use https://");
    }
    let bundle = &input.bundle_dir;
    for name in ["manifest.json", "chunks.jsonl", "LICENSE", "NOTICES"] {
        if !bundle.join(name).is_file() {
            anyhow::bail!("share bundle is missing {name}");
        }
    }
    reject_vector_files(bundle)?;

    let manifest_raw =
        fs::read_to_string(bundle.join("manifest.json")).context("read share bundle manifest")?;
    let mut source_manifest = manifest::parse_manifest(&manifest_raw)?;
    manifest::validate(&source_manifest)?;
    if source_manifest.source.chunk_count == 0 {
        anyhow::bail!("manifest chunk_count must be greater than zero");
    }

    let rows = parse_rows(&fs::read(bundle.join("chunks.jsonl"))?)?;
    if rows.len() != source_manifest.source.chunk_count as usize {
        anyhow::bail!(
            "chunks.jsonl has {} rows but manifest chunk_count is {}",
            rows.len(),
            source_manifest.source.chunk_count
        );
    }
    let mut chunks = Vec::with_capacity(rows.len());
    for (position, row) in rows.iter().enumerate() {
        if row.idx != position as u32 {
            anyhow::bail!(
                "chunks.jsonl idx {} at position {} is not contiguous",
                row.idx,
                position
            );
        }
        if row.text.trim().is_empty() {
            anyhow::bail!("chunks.jsonl row {} has empty text", position);
        }
        let chunk_type = match row.chunk_type.as_str() {
            "Code" => ChunkType::Code,
            "Info" => ChunkType::Info,
            other => anyhow::bail!(
                "chunks.jsonl row {} has invalid chunk_type {other:?}",
                position
            ),
        };
        let source_url = canonical_source_url(&input.source_url_base, &row.source_url)?;
        chunks.push(Chunk {
            idx: row.idx,
            heading_path: row.heading_path.clone(),
            source_url,
            api_version: row.api_version.clone().filter(|v| !v.is_empty()),
            chunk_type,
            text: row.text.clone(),
        });
    }

    source_manifest.docset = public_docset.clone();
    source_manifest.source.source_url = input.source_url_base.clone();
    source_manifest.source.chunk_count = chunks.len() as u32;
    manifest::validate_manifest_for_docset(&source_manifest, &public_docset)?;

    fs::create_dir_all(&input.output_dir)?;
    let normalized_path = input.output_dir.join("normalized-chunks.jsonl");
    let mut normalized = String::new();
    for chunk in &chunks {
        normalized.push_str(&serde_json::to_string(&OutputChunk {
            idx: chunk.idx,
            heading_path: &chunk.heading_path,
            source_url: &chunk.source_url,
            api_version: chunk.api_version.as_deref(),
            chunk_type: match chunk.chunk_type {
                ChunkType::Code => "Code",
                ChunkType::Info => "Info",
            },
            text: &chunk.text,
        })?);
        normalized.push('\n');
    }
    fs::write(&normalized_path, normalized)?;

    let cache_home = unique_temp_path("nowdocs-registry-build-cache");
    fs::create_dir_all(&cache_home)?;
    let previous_cache = std::env::var_os("XDG_CACHE_HOME");
    std::env::set_var("XDG_CACHE_HOME", &cache_home);
    let result = build_lance_and_copy(&cache_home, &input.output_dir, &public_docset, &chunks);
    if let Some(previous) = previous_cache {
        std::env::set_var("XDG_CACHE_HOME", previous);
    } else {
        std::env::remove_var("XDG_CACHE_HOME");
    }
    result?;

    fs::write(
        input.output_dir.join("manifest.json"),
        serde_json::to_string_pretty(&source_manifest)?,
    )?;
    fs::copy(bundle.join("LICENSE"), input.output_dir.join("LICENSE"))?;
    fs::copy(bundle.join("NOTICES"), input.output_dir.join("NOTICES"))?;
    let _ = fs::remove_dir_all(&cache_home);
    Ok(input.output_dir.clone())
}

fn build_lance_and_copy(
    cache_home: &Path,
    output: &Path,
    docset: &str,
    chunks: &[Chunk],
) -> Result<()> {
    let vectors = if chunks.is_empty() {
        Vec::new()
    } else {
        let embedder = Embedder::load()?;
        chunks
            .iter()
            .map(|chunk| embedder.embed(&chunk.text))
            .collect::<Result<Vec<_>>>()?
    };
    let store = Store::open(docset)?;
    store.insert(chunks, &vectors)?;
    drop(store);
    let source = cache_home
        .join("nowdocs/db")
        .join(format!("{docset}.lance"));
    copy_dir(&source, &output.join(format!("{docset}.lance")))
}

fn copy_dir(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source).with_context(|| format!("read {}", source.display()))? {
        let entry = entry?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

fn parse_rows(data: &[u8]) -> Result<Vec<ShareChunk>> {
    data.split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).context("parse chunks.jsonl row"))
        .collect()
}

fn validate_public_docset(docset: &str) -> Result<String> {
    let docset = input::validate_docset(docset)?;
    if !matches!(docset.as_str(), "nextjs" | "react" | "vue") {
        anyhow::bail!("public docset must be one of nextjs, react, vue");
    }
    Ok(docset)
}

fn canonical_source_url(base: &str, source: &str) -> Result<String> {
    if source.starts_with("https://") {
        return Ok(source.to_string());
    }
    let relative = source.trim_start_matches('/');
    if relative.is_empty() || relative.split('/').any(|part| part == "..") {
        anyhow::bail!("invalid relative source_url: {source:?}");
    }
    Ok(format!("{}/{}", base.trim_end_matches('/'), relative))
}

fn reject_vector_files(root: &Path) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if name.ends_with(".lance")
            || name.ends_with(".arrow")
            || name.ends_with(".npy")
            || name.ends_with(".npz")
            || name.contains("vector")
        {
            anyhow::bail!(
                "share bundle contains forbidden vector file: {}",
                entry.path().display()
            );
        }
        if entry.file_type()?.is_dir() {
            reject_vector_files(&entry.path())?;
        }
    }
    Ok(())
}

fn unique_temp_path(prefix: &str) -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{}-{stamp}", std::process::id()))
}
