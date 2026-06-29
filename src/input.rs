//! Validation for tool inputs entering the MCP boundary.
//!
//! These are the last gate before untrusted caller-supplied strings reach the
//! filesystem (docset → path) or the retrieval layer. Everything here fails
//! closed: invalid input → `Err`, never a silent default.

use std::sync::OnceLock;

use regex::Regex;

const DOCSET_RE: &str = r"^[a-z0-9._-]{1,64}$";
const QUERY_MAX_CHARS: usize = 4096;

const MAX_TOKENS_DEFAULT: u32 = 4000;
const MAX_TOKENS_CAP: u32 = 4000;
const TOP_K_DEFAULT: u32 = 5;
const TOP_K_MIN: u32 = 1;
const TOP_K_MAX: u32 = 20;

static DOCSET_REGEX: OnceLock<Regex> = OnceLock::new();

fn docset_regex() -> &'static Regex {
    DOCSET_REGEX.get_or_init(|| Regex::new(DOCSET_RE).expect("docset regex must compile"))
}

/// Validate a docset name. Allowed: `^[a-z0-9._-]{1,64}$`, with `..`
/// (path-traversal) explicitly rejected even though it matches the class.
pub fn validate_docset(s: &str) -> anyhow::Result<String> {
    if s.contains("..") {
        anyhow::bail!("docset must not contain '..': {s:?}");
    }
    if !docset_regex().is_match(s) {
        anyhow::bail!("docset must match {DOCSET_RE:?}: {s:?}");
    }
    Ok(s.to_string())
}

/// Validate a query string: non-empty, ≤ 4096 chars.
pub fn validate_query(s: &str) -> anyhow::Result<String> {
    if s.is_empty() {
        anyhow::bail!("query must be non-empty");
    }
    if s.chars().count() > QUERY_MAX_CHARS {
        anyhow::bail!("query exceeds {QUERY_MAX_CHARS} chars");
    }
    Ok(s.to_string())
}

/// Resolve the retrieval `max_tokens` budget.
///
/// `None` or `0` → default (4000). `Some(v)` → `min(v, cap)` where cap = 4000.
/// The signature returns `u32` (locked in 1a) so `0` cannot be a hard error;
/// it is treated as "unset" and falls back to the default.
pub fn resolve_max_tokens(n: Option<u32>) -> u32 {
    match n {
        None | Some(0) => MAX_TOKENS_DEFAULT,
        Some(v) => v.min(MAX_TOKENS_CAP),
    }
}

/// Resolve `top_k`, clamped to `[1, 20]`. `None` → 5.
pub fn resolve_top_k(n: Option<u32>) -> u32 {
    match n {
        None => TOP_K_DEFAULT,
        Some(v) => v.clamp(TOP_K_MIN, TOP_K_MAX),
    }
}
