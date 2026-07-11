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
    /// M25: optional upstream base URL for per-chunk `source_url`. When set,
    /// each chunk's `source_url` is `{base}/{relative_path}` so canonical
    /// registry docsets carry traceable upstream URLs. When `None` (local
    /// private docs), the relative markdown path is kept verbatim.
    pub source_url_base: Option<String>,
}

impl Default for IngestMeta {
    /// Defaults preserve the pre-flag behavior: MIT license, empty source
    /// fields, no per-chunk base URL. CC-BY-4.0 callers must set `attribution`
    /// or `manifest::validate` will reject the result.
    fn default() -> Self {
        Self {
            license: "MIT".to_string(),
            copyright_holder: String::new(),
            attribution: String::new(),
            source_url: String::new(),
            entry_url: String::new(),
            source_url_base: None,
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
            // M25: canonical docsets get a traceable upstream URL; local private
            // docs keep the relative markdown path.
            c.source_url = source_url_for_chunk(meta.source_url_base.as_deref(), &rel);
        }
        chunks.extend(file_chunks);
    }

    // Reassign global idx (chunker's idx is per-file from 0).
    for (i, c) in chunks.iter_mut().enumerate() {
        c.idx = i as u32;
    }

    // Read the source repo's LICENSE text (if present) WITHOUT publishing it
    // yet. It is only written to the cache after the store+manifest update
    // succeeds, so an embedder/validation failure preserves the previous
    // license sidecar too (S4 fail-safe). We copy what upstream ships, we do
    // not regenerate text from the SPDX id; no LICENSE file → nothing stashed.
    let license_text = find_license_text(dir);

    // S4: compute the replacement vectors BEFORE wiping the active store, so an
    // embedder/model failure cannot turn a recoverable ingest into data loss.
    // This mirrors `rebuild_docset`, which carries an explicit comment warning
    // against the old wipe-before-embed order. Empty dirs skip embedder load.
    let vectors: Result<Vec<Vec<f32>>> = if chunks.is_empty() {
        Ok(Vec::new())
    } else {
        embed_chunks(&chunks)
    };

    let stats = ingest_chunks(&docset, meta, chunks, files, vectors)?;

    // S4: publish the license sidecar only after the store+manifest succeed,
    // so a failed ingest cannot pair the previous store/manifest with a new
    // (or absent) license that `nowdocs share` would then carry.
    if let Some(text) = license_text {
        std::fs::write(cache::license_text_path(&docset), text)?;
    }

    Ok(stats)
}

/// Load the embedder and embed every chunk. Isolated so `ingest_dir` can compute
/// vectors before wiping the store (S4) and so tests can inject a failure.
fn embed_chunks(chunks: &[Chunk]) -> Result<Vec<Vec<f32>>> {
    let emb = embedder::Embedder::load()?;
    chunks
        .iter()
        .map(|c| emb.embed(&c.text))
        .collect::<Result<_>>()
}

/// Write a pre-chunked, pre-embedded docset to the store + manifest.
///
/// `vectors` is a `Result` so an embedder failure (computed by the caller) is
/// observed AFTER manifest validation but BEFORE the store is wiped — the
/// fail-safe ordering that preserves the previous store on embed errors (S4).
/// The manifest write is atomic (tmp + rename, M5).
fn ingest_chunks(
    docset: &str,
    meta: &IngestMeta,
    chunks: Vec<Chunk>,
    files: u32,
    vectors: Result<Vec<Vec<f32>>>,
) -> Result<IngestStats> {
    // Build + validate the manifest BEFORE touching the store. Invalid metadata
    // (e.g. CC-BY-4.0 without --attribution) must fail fast here, otherwise
    // Store::open + insert would leave an orphan `.lance` directory with no
    // manifest — list-installed would then report a broken docset and later
    // search/share would fail. chunk_count is already known at this point.
    let manifest = build_manifest(docset, chunks.len() as u32, meta);
    manifest::validate(&manifest)?;

    // Surface an embedder failure now — after validation, before the wipe — so
    // the previous store (if any) is preserved.
    let vectors = vectors?;

    // Ingest is full-rebuild semantics: the manifest (below) and the stashed
    // license text (above) are overwritten on every run, so the lance table
    // must be too. Without this wipe, re-ingesting the same docset appends to
    // the existing table via `Store::insert` (`table.add`), doubling every
    // chunk_idx and polluting hybrid search with duplicate hits. Tests miss
    // this because every ingest test isolates with a fresh `tempdir()` cache.
    wipe_store(docset)?;

    // Insert. Empty dir still opens (empty) store.
    if !chunks.is_empty() {
        let store = Store::open(docset)?;
        store.insert(&chunks, &vectors)?;
    } else {
        let _ = Store::open(docset)?;
    }

    // M5: atomic manifest write (write tmp, then rename into place) so a crash
    // mid-write can never publish a half-written manifest.
    let manifest_path = cache::manifest_path(docset);
    let tmp = manifest_path.with_extension("tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(&manifest)?)?;
    std::fs::rename(&tmp, &manifest_path)?;

    Ok(IngestStats {
        files,
        chunks: chunks.len() as u32,
    })
}

/// Rebuild an existing docset from the text already stored in LanceDB.
///
/// This is the one-command escape hatch for schema/embedder changes: dump the
/// sanitized text+metadata rows, remove the old physical table, re-embed with
/// the current model, recreate the table with the current Arrow schema, and
/// rewrite the manifest with current schema/embedder provenance.
pub fn rebuild_docset(docset: &str) -> Result<IngestStats> {
    let docset = input::validate_docset(docset)?;
    cache::ensure_layout()?;

    let manifest_path = cache::manifest_path(&docset);
    let db_path = cache::db_path(&docset);
    if !manifest_path.is_file() || !db_path.exists() {
        anyhow::bail!(
            "docset `{}` is not installed; install or ingest it before running `nowdocs rebuild {}`",
            docset,
            docset
        );
    }

    let raw = std::fs::read_to_string(&manifest_path).ok();
    let old_manifest = raw
        .as_deref()
        .and_then(|r| manifest::parse_manifest(r).ok());

    let store = Store::open(&docset)?;
    let chunks = store.dump_chunks()?;
    drop(store);

    // Rebuild must be non-destructive on embedder/model failures. Compute the
    // replacement vectors before deleting the active Lance table so an offline
    // or corrupt model cache does not turn a recoverable rebuild attempt into
    // data loss.
    let vectors = if chunks.is_empty() {
        Vec::new()
    } else {
        let emb = embedder::Embedder::load()?;
        chunks
            .iter()
            .map(|c| emb.embed(&c.text))
            .collect::<Result<Vec<_>>>()?
    };

    wipe_store(&docset)?;

    if !chunks.is_empty() {
        let store = Store::open(&docset)?;
        store.insert(&chunks, &vectors)?;
    } else {
        let _ = Store::open(&docset)?;
    }

    let meta = raw
        .as_deref()
        .and_then(|r| {
            let v: serde_json::Value = serde_json::from_str(r).ok()?;
            Some(IngestMeta {
                license: v["legal"]["license"].as_str().unwrap_or("MIT").to_string(),
                copyright_holder: v["legal"]["copyright_holder"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
                attribution: v["legal"]["attribution"].as_str().unwrap_or("").to_string(),
                source_url: v["source"]["source_url"].as_str().unwrap_or("").to_string(),
                entry_url: v["source"]["entry_url"].as_str().unwrap_or("").to_string(),
                source_url_base: None,
            })
        })
        .or_else(|| {
            old_manifest.as_ref().map(|m| IngestMeta {
                license: m.legal.license.clone(),
                copyright_holder: m.legal.copyright_holder.clone(),
                attribution: m.legal.attribution.clone(),
                source_url: m.source.source_url.clone(),
                entry_url: m.source.entry_url.clone(),
                source_url_base: None,
            })
        })
        .unwrap_or_default();
    let mut new_manifest = build_manifest(&docset, chunks.len() as u32, &meta);
    if let Some(old) = old_manifest {
        new_manifest.doc_version = old.doc_version;
        new_manifest.refresh_strategy = old.refresh_strategy;
    }
    manifest::validate(&new_manifest)?;
    let manifest_file_path = cache::manifest_path(&docset);
    let tmp = manifest_file_path.with_extension("tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(&new_manifest)?)?;
    std::fs::rename(&tmp, &manifest_file_path)?;

    Ok(IngestStats {
        files: 0,
        chunks: chunks.len() as u32,
    })
}

/// Remove an existing docset's lance table so the next `Store::open` recreates
/// it empty. Ingest is full-rebuild semantics — see `ingest_dir`. A missing
/// path is a no-op (first ingest); an existing path is removed whether it is a
/// directory (the normal `.lance` case) or a stray file.
pub fn wipe_store(docset: &str) -> Result<()> {
    let path = cache::db_path(docset);
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        std::fs::remove_dir_all(&path)
            .with_context(|| format!("failed to wipe stale store at {}", path.display()))?;
    } else {
        std::fs::remove_file(&path)
            .with_context(|| format!("failed to remove stale store file at {}", path.display()))?;
    }
    Ok(())
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

/// Compose a chunk's `source_url` from an optional upstream base and the
/// relative markdown path (M25).
///
/// - `Some(base)` → `"{base_trimmed}/{rel}"`, with any trailing `/` on the base
///   stripped so callers don't have to normalize it.
/// - `None` → the relative path verbatim (local private docs).
///
/// Pure function so the URL-contract behavior is unit-testable offline without
/// running the embedder or touching the store.
fn source_url_for_chunk(base: Option<&str>, rel: &str) -> String {
    match base {
        Some(b) => format!("{}/{}", b.trim_end_matches('/'), rel),
        None => rel.to_string(),
    }
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
            chunk_size_tokens: chunker::default_config().target_tokens,
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
        assert_eq!(find_license_text(d.path()), Some("GPL body\n".to_string()));
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

    // S4: an embedder/vectors failure must NOT wipe the existing store, because
    // the wipe happens after vector computation. We inject the failure through
    // `ingest_chunks`' `vectors: Result<...>` seam (no real embedder needed).
    #[test]
    fn ingest_failed_embedder_load_preserves_store() {
        use crate::chunker::{Chunk, ChunkType};
        use crate::store::Store;

        let d = tempdir().unwrap();
        unsafe { std::env::set_var("XDG_CACHE_HOME", d.path()) };
        crate::cache::ensure_layout().unwrap();

        let docset = "embed_fail_preserve";

        // Pre-populate an existing store with 2 rows (no embedder needed).
        let store = Store::open(docset).unwrap();
        let seed = vec![
            Chunk {
                idx: 0,
                heading_path: "A".into(),
                source_url: "a.md".into(),
                api_version: None,
                chunk_type: ChunkType::Info,
                text: "alpha".into(),
            },
            Chunk {
                idx: 1,
                heading_path: "B".into(),
                source_url: "b.md".into(),
                api_version: None,
                chunk_type: ChunkType::Info,
                text: "beta".into(),
            },
        ];
        let seed_vecs: Vec<Vec<f32>> = seed.iter().map(|_| vec![0.0f32; 512]).collect();
        store.insert(&seed, &seed_vecs).unwrap();
        drop(store);

        // Attempt to ingest the same docset but force the vectors step to fail.
        let incoming = vec![
            Chunk {
                idx: 0,
                heading_path: "C".into(),
                source_url: "c.md".into(),
                api_version: None,
                chunk_type: ChunkType::Info,
                text: "gamma".into(),
            },
            Chunk {
                idx: 1,
                heading_path: "D".into(),
                source_url: "d.md".into(),
                api_version: None,
                chunk_type: ChunkType::Info,
                text: "delta".into(),
            },
        ];
        let res = super::ingest_chunks(
            docset,
            &super::IngestMeta::default(),
            incoming,
            2,
            Err(anyhow::anyhow!("simulated embedder failure")),
        );
        assert!(res.is_err(), "embedder failure must propagate");

        // The previous store must be intact (wipe never ran).
        let store = Store::open(docset).unwrap();
        let rows = store.dump_chunks().unwrap();
        assert_eq!(
            rows.len(),
            2,
            "existing store must be preserved on embedder failure"
        );
        assert_eq!(rows[0].text, "alpha");
        assert_eq!(rows[1].text, "beta");
        // And no manifest was published for the failed ingest.
        assert!(
            !crate::cache::manifest_path(docset).is_file(),
            "failed ingest must not publish a manifest"
        );
    }

    // --- source_url_for_chunk (M25) ---
    //
    // Spec test names (`ingest_with_source_url_base_writes_full_url` /
    // `ingest_without_source_url_base_writes_relative_path`) are asserted against
    // the pure helper that `ingest_dir` calls, so the contract is covered offline
    // without loading the embedder.

    #[test]
    fn ingest_with_source_url_base_writes_full_url() {
        assert_eq!(
            super::source_url_for_chunk(Some("https://example.com/docs"), "guide/start.md"),
            "https://example.com/docs/guide/start.md"
        );
    }

    #[test]
    fn ingest_without_source_url_base_writes_relative_path() {
        assert_eq!(
            super::source_url_for_chunk(None, "guide/start.md"),
            "guide/start.md"
        );
    }

    #[test]
    fn source_url_for_chunk_trims_trailing_slash_on_base() {
        assert_eq!(
            super::source_url_for_chunk(Some("https://example.com/docs/"), "guide/start.md"),
            "https://example.com/docs/guide/start.md"
        );
    }
}
