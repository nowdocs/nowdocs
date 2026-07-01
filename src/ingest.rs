//! Ingest pipeline: markdown directory -> chunks -> embeddings -> store + manifest.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::cache;
use crate::chunker::{self, Chunk};
use crate::embedder;
use crate::input;
use crate::manifest::{
    self, EmbedderSpec, LegalSpec, Manifest, RefreshSpec, RetrievalSpec, SourceSpec,
};
use crate::store::Store;

#[derive(Debug)]
pub struct IngestStats {
    pub files: u32,
    pub chunks: u32,
}

/// Legal + source metadata for an ingested docset.
///
/// Absorbs what `seed-crates/tmp/patch_manifest.py` used to do as a post-hoc
/// Python patch: the caller passes license/copyright/attribution/source fields
/// at ingest time so the manifest is correct on first write, with no external
/// patching step. `scraped_at` is auto-filled with today's UTC date (no chrono
/// dependency — see `today_iso`).
#[derive(Clone)]
pub struct IngestMeta {
    pub license: String,
    pub copyright_holder: String,
    pub attribution: String,
    pub source_url: String,
    pub entry_url: String,
}

impl Default for IngestMeta {
    /// Defaults preserve the pre-flag behavior: MIT license, empty source
    /// fields. CC-BY-4.0 callers must set `attribution` or `manifest::validate`
    /// will reject the result.
    fn default() -> Self {
        Self {
            license: "MIT".to_string(),
            copyright_holder: String::new(),
            attribution: String::new(),
            source_url: String::new(),
            entry_url: String::new(),
        }
    }
}

/// Ingest a directory of markdown files into a docset store.
pub fn ingest_dir(dir: &Path, docset: &str, meta: &IngestMeta) -> Result<IngestStats> {
    let docset = input::validate_docset(docset)?;
    cache::ensure_layout()?;

    // Collect chunks across all md files.
    let mut chunks: Vec<Chunk> = Vec::new();
    let mut files: u32 = 0;
    for entry in walk_md(dir)? {
        files += 1;
        let md =
            std::fs::read_to_string(&entry).with_context(|| format!("read {}", entry.display()))?;
        let mut file_chunks = chunker::chunk_markdown(&md, &chunker::default_config());
        let rel = entry
            .strip_prefix(dir)
            .unwrap_or(&entry)
            .to_string_lossy()
            .to_string();
        for c in &mut file_chunks {
            c.source_url = rel.clone();
        }
        chunks.extend(file_chunks);
    }

    // Reassign global idx (chunker's idx is per-file from 0).
    for (i, c) in chunks.iter_mut().enumerate() {
        c.idx = i as u32;
    }

    // Stash the source repo's LICENSE text (if present) so `nowdocs share` can
    // carry the verbatim upstream license in the bundle. This is the ground
    // truth — we copy what upstream ships, we do not regenerate text from the
    // SPDX id. No LICENSE file → nothing stashed; share then emits NOTICE only.
    if let Some(text) = find_license_text(dir) {
        std::fs::write(cache::license_text_path(&docset), text)?;
    }

    // Build + validate the manifest BEFORE touching the store. Invalid metadata
    // (e.g. CC-BY-4.0 without --attribution) must fail fast here, otherwise
    // Store::open + insert would leave an orphan `.lance` directory with no
    // manifest — list-installed would then report a broken docset and later
    // search/share would fail. chunk_count is already known at this point.
    let manifest = build_manifest(&docset, chunks.len() as u32, meta);
    manifest::validate(&manifest)?;

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

    // Store written successfully — persist the pre-validated manifest.
    std::fs::write(
        cache::manifest_path(&docset),
        serde_json::to_string_pretty(&manifest)?,
    )?;

    Ok(IngestStats {
        files,
        chunks: chunks.len() as u32,
    })
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
            } else if path.extension().is_some_and(|e| e == "md") {
                out.push(path);
            }
        }
    }
    out.sort();
    Ok(out)
}

/// Locate the upstream license text in the ingest `dir` root. Returns the
/// verbatim contents of the first match, or `None`.
///
/// Matches `LICENSE` / `LICENSE.md` / `LICENSE.txt` / `COPYING` / …
/// (case-insensitive on the stem), preferring `LICENSE*` over `COPYING*`.
/// Only scans the dir root — a docs source rarely nests its license, and
/// recursing would risk pulling in a vendored `node_modules/LICENSE`.
fn find_license_text(dir: &Path) -> Option<String> {
    let mut license: Option<PathBuf> = None;
    let mut copying: Option<PathBuf> = None;
    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
            let stem = name.trim_end_matches(".md").trim_end_matches(".txt");
            if stem == "license" && license.is_none() {
                license = Some(path);
            } else if stem == "copying" && copying.is_none() {
                copying = Some(path);
            }
        }
    }
    std::fs::read_to_string(license.or(copying)?).ok()
}

/// Build the v1 manifest with locked embedder provenance.
fn build_manifest(docset: &str, chunk_count: u32, meta: &IngestMeta) -> Manifest {
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
            entry_url: meta.entry_url.clone(),
            source_url: meta.source_url.clone(),
            scraped_at: today_iso(),
            chunk_count,
        },
        legal: LegalSpec {
            license: meta.license.clone(),
            copyright_holder: meta.copyright_holder.clone(),
            attribution: meta.attribution.clone(),
        },
        refresh_strategy: RefreshSpec {
            tier: "community".to_string(),
            auto_days: 0,
        },
    }
}

/// Today's UTC date as `YYYY-MM-DD`, using only `std` (no chrono dependency —
/// adding a date crate would cross the Cargo.toml red line).
fn today_iso() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = (secs / 86400) as i64;
    civil_from_days(days)
}

/// Convert days-since-Unix-epoch to `YYYY-MM-DD`. Howard Hinnant's algorithm
/// (civil_from_days); valid proleptic Gregorian, UTC.
fn civil_from_days(z_in: i64) -> String {
    let z = z_in + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", y, m, d)
}

#[cfg(test)]
mod tests {
    use super::civil_from_days;

    #[test]
    fn test_civil_from_days_epoch() {
        // 1970-01-01 is day 0.
        assert_eq!(civil_from_days(0), "1970-01-01");
    }

    #[test]
    fn test_civil_from_days_known_dates() {
        // 2024-01-01 = day 19723 (54 years × 365 + 13 leap days).
        assert_eq!(civil_from_days(19_723), "2024-01-01");
        // 2024 is a leap year; Dec 1 = 19723 + 335.
        assert_eq!(civil_from_days(20_058), "2024-12-01");
        assert_eq!(civil_from_days(20_088), "2024-12-31"); // leap-year edge
        assert_eq!(civil_from_days(20_089), "2025-01-01");
    }

    #[test]
    fn test_civil_from_days_2026() {
        // 2026-01-01 = 20089 + 365 (2025 is not a leap year) = 20454.
        assert_eq!(civil_from_days(20_454), "2026-01-01");
        // 2026-04-15 = 20454 + (31+28+31) + 14 = 20558.
        assert_eq!(civil_from_days(20_558), "2026-04-15");
    }

    // --- find_license_text: locate the upstream license file in the ingest dir ---

    use super::find_license_text;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_find_license_text_finds_license() {
        let d = tempdir().unwrap();
        fs::write(d.path().join("LICENSE"), "MIT license body\n").unwrap();
        assert_eq!(
            find_license_text(d.path()),
            Some("MIT license body\n".to_string())
        );
    }

    #[test]
    fn test_find_license_text_finds_license_md() {
        let d = tempdir().unwrap();
        fs::write(d.path().join("LICENSE.md"), "body\n").unwrap();
        assert_eq!(find_license_text(d.path()), Some("body\n".to_string()));
    }

    #[test]
    fn test_find_license_text_case_insensitive() {
        let d = tempdir().unwrap();
        fs::write(d.path().join("license"), "lowercase name\n").unwrap();
        assert_eq!(
            find_license_text(d.path()),
            Some("lowercase name\n".to_string())
        );
    }

    #[test]
    fn test_find_license_text_finds_copying_when_no_license() {
        let d = tempdir().unwrap();
        fs::write(d.path().join("COPYING"), "GPL body\n").unwrap();
        assert_eq!(
            find_license_text(d.path()),
            Some("GPL body\n".to_string())
        );
    }

    #[test]
    fn test_find_license_text_prefers_license_over_copying() {
        let d = tempdir().unwrap();
        fs::write(d.path().join("LICENSE"), "MIT\n").unwrap();
        fs::write(d.path().join("COPYING"), "GPL\n").unwrap();
        assert_eq!(find_license_text(d.path()), Some("MIT\n".to_string()));
    }

    #[test]
    fn test_find_license_text_none_when_absent() {
        let d = tempdir().unwrap();
        fs::write(d.path().join("README.md"), "readme\n").unwrap();
        assert_eq!(find_license_text(d.path()), None);
    }
}
