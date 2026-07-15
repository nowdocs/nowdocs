//! Registry provenance receipts.
//!
//! A receipt records that a docset was installed from a verified Registry
//! package. It is written atomically only after a successful promotion, while
//! the install lock is held. Local ingests never create receipts.

use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;

use crate::cache;
use crate::input;
use crate::manifest;
use crate::registry::RegistryPackage;

const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryReceipt {
    schema_version: u32,
    pub docset: String,
    pub package_version: String,
    pub package_sha256: String,
    pub manifest_sha256: String,
}

/// Path to the receipt file for a docset.
pub fn receipt_path(docset: &str) -> PathBuf {
    cache::registry_receipts_root().join(format!("{docset}.json"))
}

/// Record a receipt after a verified package promotion.
///
/// Validates the docset name, reads the current promoted manifest, verifies
/// the doc_version matches the package version, hashes the manifest bytes,
/// and atomically writes the receipt JSON.
pub fn record_after_promotion(package: &RegistryPackage) -> Result<()> {
    let docset = input::validate_docset(&package.docset)
        .context("registry provenance receipt: invalid docset")?;

    let manifest_path = cache::manifest_path(&docset);
    let manifest_raw = std::fs::read_to_string(&manifest_path)
        .context("registry provenance receipt: read promoted manifest")?;

    let m = manifest::parse_manifest(&manifest_raw)
        .context("registry provenance receipt: parse manifest")?;

    if m.doc_version != package.version {
        anyhow::bail!(
            "registry provenance receipt: manifest version {} does not match package version {}",
            m.doc_version,
            package.version
        );
    }

    let manifest_sha256 = sha256_hex(manifest_raw.as_bytes());

    let receipt = RegistryReceipt {
        schema_version: SCHEMA_VERSION,
        docset: docset.to_string(),
        package_version: package.version.clone(),
        package_sha256: package.sha256.clone(),
        manifest_sha256,
    };

    let root = cache::registry_receipts_root();
    std::fs::create_dir_all(&root)
        .context("registry provenance receipt: create receipts directory")?;

    let json = serde_json::to_string(&receipt)?;
    let mut tmp =
        NamedTempFile::new_in(&root).context("registry provenance receipt: create temp file")?;
    tmp.write_all(json.as_bytes())?;
    tmp.flush()?;
    tmp.as_file().sync_all()?;
    tmp.persist(receipt_path(&docset))
        .map_err(|e| anyhow::Error::new(e.error))
        .context("registry provenance receipt: persist receipt")?;

    Ok(())
}

/// Remove a receipt for a docset. Called during uninstall.
/// A missing receipt is not an error.
pub fn remove(docset: &str) -> Result<()> {
    let path = receipt_path(docset);
    if path.is_file() {
        std::fs::remove_file(&path).context("registry provenance receipt: remove receipt")?;
    }
    Ok(())
}

/// Load all receipts whose current manifest is healthy, version-equal, and
/// hash-equal. Malformed or stale files are silently excluded.
pub fn load_matching_installed() -> Vec<RegistryReceipt> {
    let root = cache::registry_receipts_root();
    let mut result = Vec::new();

    let entries = match std::fs::read_dir(&root) {
        Ok(e) => e,
        Err(_) => return result,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let raw = match std::fs::read_to_string(&path) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let receipt: RegistryReceipt = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(_) => continue,
        };
        if receipt.schema_version != SCHEMA_VERSION {
            continue;
        }
        if input::validate_docset(&receipt.docset).is_err() {
            continue;
        }
        if !matches_receipt(&receipt) {
            continue;
        }
        result.push(receipt);
    }

    result
}

/// Check if a receipt matches the current installed state.
fn matches_receipt(receipt: &RegistryReceipt) -> bool {
    let manifest_path = cache::manifest_path(&receipt.docset);
    let manifest_raw = match std::fs::read_to_string(&manifest_path) {
        Ok(r) => r,
        Err(_) => return false,
    };
    let m = match manifest::parse_manifest(&manifest_raw) {
        Ok(m) => m,
        Err(_) => return false,
    };
    if manifest::validate(&m).is_err() {
        return false;
    }
    if m.doc_version != receipt.package_version {
        return false;
    }
    let actual_hash = sha256_hex(manifest_raw.as_bytes());
    actual_hash == receipt.manifest_sha256
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    let mut s = String::with_capacity(64);
    for b in result.iter() {
        use std::fmt::Write;
        let _ = write!(s, "{b:02x}");
    }
    s
}
