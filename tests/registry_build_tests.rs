use std::fs;
use std::path::Path;

use nowdocs::registry_build::{build_registry_release, RegistryReleaseBuild};
use tempfile::TempDir;

fn valid_manifest(chunk_count: usize) -> String {
    format!(
        r#"{{
  "docset": "nextjs-docs",
  "doc_version": "1.0.0",
  "nowdocs_schema_version": 1,
  "embedder": {{
    "model_id": "jinaai/jina-embeddings-v2-small-en",
    "model_version": "2",
    "model_revision": "pinned-revision",
    "model_sha256": "pinned-sha256",
    "vector_dim": 512,
    "engine": "candle",
    "dtype": "f16"
  }},
  "retrieval": {{ "tokenizer": "default", "chunk_size_tokens": 512, "window_tokens": 2048 }},
  "source": {{ "entry_url": "https://docs.example.test", "source_url": "https://docs.example.test", "scraped_at": "2026-07-12", "chunk_count": {chunk_count} }},
  "legal": {{ "license": "MIT", "copyright_holder": "Example", "attribution": "" }},
  "refresh_strategy": {{ "tier": "manual", "auto_days": 0 }}
}}"#
    )
}

fn make_share_bundle(count: usize) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("manifest.json"), valid_manifest(count)).unwrap();
    fs::write(dir.path().join("LICENSE"), "MIT license\n").unwrap();
    fs::write(dir.path().join("NOTICES"), "Example notice\n").unwrap();
    let rows = (0..count)
        .map(|idx| {
            format!(
                r#"{{"idx":{idx},"heading_path":"Guide","source_url":"guide/install.md","api_version":"","chunk_type":"Info","text":"installation configuration example {idx}"}}"#
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(dir.path().join("chunks.jsonl"), format!("{rows}\n")).unwrap();
    dir
}

fn build_input(
    bundle_dir: &Path,
    public_docset: &str,
    source_url_base: &str,
) -> RegistryReleaseBuild {
    RegistryReleaseBuild {
        bundle_dir: bundle_dir.to_path_buf(),
        output_dir: bundle_dir.join("output"),
        public_docset: public_docset.to_string(),
        source_url_base: source_url_base.to_string(),
    }
}

fn write_manifest_chunk_count(bundle_dir: &Path, count: usize) {
    fs::write(bundle_dir.join("manifest.json"), valid_manifest(count)).unwrap();
}

#[test]
fn release_builder_rejects_vector_files_in_share_bundle() {
    let fixture = make_share_bundle(1);
    fs::create_dir(fixture.path().join("payload.lance")).unwrap();
    let input = build_input(fixture.path(), "nextjs", "https://docs.example.test");
    assert!(build_registry_release(&input).is_err());
}

#[test]
fn release_builder_rejects_manifest_chunk_count_mismatch() {
    let fixture = make_share_bundle(2);
    write_manifest_chunk_count(fixture.path(), 1);
    let input = build_input(fixture.path(), "nextjs", "https://docs.example.test");
    assert!(build_registry_release(&input).is_err());
}

#[test]
#[ignore = "loads the pinned embedder unless a model cache is already present"]
fn release_builder_rewrites_relative_chunk_sources_to_canonical_urls() {
    let fixture = make_share_bundle(1);
    let input = build_input(fixture.path(), "nextjs", "https://docs.example.test/base");
    let output = build_registry_release(&input).unwrap();
    assert!(fs::read_to_string(output.join("normalized-chunks.jsonl"))
        .unwrap()
        .contains("https://docs.example.test/base/guide/install.md"));
}

#[test]
#[ignore = "loads the pinned embedder and builds a Lance release artifact"]
fn release_builder_creates_searchable_lance_table() {
    let fixture = make_share_bundle(2);
    let input = build_input(fixture.path(), "nextjs", "https://docs.example.test");
    let output = build_registry_release(&input).unwrap();
    assert!(output.join("nextjs.lance").is_dir());
    assert!(output.join("manifest.json").is_file());
    assert!(output.join("LICENSE").is_file());
    assert!(output.join("NOTICES").is_file());
    assert!(output
        .join("nextjs.lance")
        .read_dir()
        .unwrap()
        .next()
        .is_some());
}
