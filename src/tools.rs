//! MCP tool handlers (nowdocs_search / nowdocs_list).

use serde_json::{json, Value};

use crate::{cache, input, retrieve, sanitize};

/// JSON-RPC error codes.
const ERR_INVALID_PARAMS: i64 = -32602;
const ERR_METHOD_NOT_FOUND: i64 = -32601;

/// Business-level error classification for tool execution.
///
/// These are surfaced to the MCP client as a tool result with `isError: true`
/// plus an actionable hint — never as a raw JSON-RPC error, and never with the
/// internal error chain leaked to the LLM.
///
/// Classification uses the downcastable sentinel error types defined in
/// `retrieve.rs` (N2, a1-mcp-error-contract §3.1): `NotInstalled` via a
/// deterministic pre-flight manifest check plus the `DocsetNotInstalled`
/// sentinel as defense-in-depth, and the rest via
/// `anyhow::Error::downcast_ref` — no string matching on the error chain.
#[derive(Debug)]
#[allow(dead_code)] // QueryInvalid + detail fields are spec-mandated, not yet wired.
enum ToolError {
    NotInstalled { docset: String },
    ModelUnavailable { detail: String },
    StoreCorrupt { docset: String, detail: String },
    QueryInvalid { detail: String },
    Internal { detail: String },
}

impl ToolError {
    /// Client-facing actionable hint. Deliberately omits `detail` (internal).
    fn hint(&self) -> String {
        match self {
            ToolError::NotInstalled { docset } => format!(
                "docset '{docset}' is not installed. run: nowdocs install {docset}"
            ),
            ToolError::ModelUnavailable { .. } => {
                "the embedder model is unavailable. run `nowdocs doctor` to diagnose, or reinstall the docset to fetch the model.".to_string()
            }
            ToolError::StoreCorrupt { docset, .. } => format!(
                "the docset '{docset}' store is corrupt. run `nowdocs rebuild {docset}` to re-index, or `nowdocs install {docset}` to reinstall."
            ),
            ToolError::QueryInvalid { detail } => format!("invalid query: {detail}"),
            ToolError::Internal { .. } => {
                "an internal error occurred while searching. run `nowdocs doctor` to diagnose.".to_string()
            }
        }
    }

    /// Render as an MCP tool result with `isError: true`.
    fn to_tool_result(&self) -> Value {
        json!({
            "content": [{ "type": "text", "text": self.hint() }],
            "isError": true,
        })
    }
}

/// Classify a `retrieve::search` error into a [`ToolError`] variant.
///
/// N2: pure sentinel downcast — zero string matching. `NotInstalled` is
/// normally produced by the pre-flight manifest check in `handle_search`; the
/// `DocsetNotInstalled` downcast here is defense-in-depth for the race between
/// that check and `retrieve::search`'s manifest read.
fn classify_error(err: &anyhow::Error, docset: &str) -> ToolError {
    if err.downcast_ref::<retrieve::DocsetNotInstalled>().is_some() {
        return ToolError::NotInstalled {
            docset: docset.to_string(),
        };
    }
    if err.downcast_ref::<retrieve::EmbedderLoadError>().is_some() {
        return ToolError::ModelUnavailable {
            detail: err.to_string(),
        };
    }
    if err.downcast_ref::<retrieve::StoreError>().is_some() {
        return ToolError::StoreCorrupt {
            docset: docset.to_string(),
            detail: err.to_string(),
        };
    }
    ToolError::Internal {
        detail: err.to_string(),
    }
}

/// Dispatch an MCP tool call by name (local-only, no reranker).
///
/// Returns a JSON-RPC `result` on success, or an `error` object on failure.
pub fn handle_call(name: &str, args: Value) -> Value {
    handle_call_with_reranker(name, args, None)
}

/// Dispatch an MCP tool call by name with an optional reranker.
///
/// Returns a JSON-RPC `result` on success, or an `error` object on failure.
pub(crate) fn handle_call_with_reranker(
    name: &str,
    args: Value,
    reranker: Option<&dyn crate::rerank::Reranker>,
) -> Value {
    match name {
        "nowdocs_search" => handle_search(args, reranker),
        "nowdocs_list" => handle_list(),
        other => err_response(ERR_METHOD_NOT_FOUND, &format!("unknown tool: {other}")),
    }
}

fn handle_search(args: Value, reranker: Option<&dyn crate::rerank::Reranker>) -> Value {
    // Extract and validate inputs.
    let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
    let docset = args.get("docset").and_then(|v| v.as_str()).unwrap_or("");
    let max_tokens = args
        .get("max_tokens")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);
    let top_k = args.get("top_k").and_then(|v| v.as_u64()).map(|v| v as u32);

    if let Err(e) = input::validate_query(query) {
        return err_response(ERR_INVALID_PARAMS, &format!("invalid query: {e}"));
    }
    if let Err(e) = input::validate_docset(docset) {
        return err_response(ERR_INVALID_PARAMS, &format!("invalid docset: {e}"));
    }

    // Pre-flight: a missing docset is a business condition, not a parameter
    // error, so it returns `isError: true` with an install hint rather than a
    // JSON-RPC -32602. We detect it by manifest presence (deterministic) rather
    // than string-matching the retrieval error chain.
    if !cache::manifest_path(docset).exists() {
        return ToolError::NotInstalled {
            docset: docset.to_string(),
        }
        .to_tool_result();
    }

    // Run retrieval pipeline.
    let search_result =
        match retrieve::search_with_reranker(docset, query, max_tokens, top_k, reranker) {
            Ok(r) => r,
            Err(e) => {
                let te = classify_error(&e, docset);
                eprintln!("nowdocs_search failed (docset={docset}): {te:?} — {e:#}");
                return te.to_tool_result();
            }
        };

    // Sanitize each chunk.
    // OQ14: `score` is intentionally NOT exposed to the LLM in v1. RRF/BM25/vector
    // scores are cross-channel incomparable; exposing them would let clients
    // over-trust one channel. Deferred to v2.
    let chunks: Vec<Value> = search_result
        .chunks
        .iter()
        .map(|c| {
            json!({
                "chunk_idx": c.chunk_idx,
                "heading_path": sanitize::sanitize_chunk(&c.heading_path),
                "source_url": sanitize::sanitize_metadata(&c.source_url),
                "api_version": c.api_version.as_deref().map(sanitize::sanitize_metadata),
                "chunk_type": format!("{:?}", c.chunk_type),
                "text": sanitize::sanitize_chunk(&c.text),
            })
        })
        .collect();

    // Text fallback: render each chunk's content for clients that don't support structuredContent.
    let mut fallback = String::new();
    for (i, chunk) in search_result.chunks.iter().enumerate() {
        if i > 0 {
            fallback.push_str("\n\n---\n\n");
        }
        fallback.push_str(&format!(
            "## {} ({})\n{}",
            sanitize::sanitize_chunk(&chunk.heading_path),
            sanitize::sanitize_metadata(&chunk.source_url),
            sanitize::sanitize_chunk(&chunk.text)
        ));
    }
    fallback.push_str(&format!(
        "\n\n---\n{} chunks, {} tokens, truncated={}",
        chunks.len(),
        search_result.tokens_returned,
        search_result.truncated
    ));

    // P1: structuredContent is a top-level result field, not inside content array.
    // P2: text fallback includes actual chunk content, not just statistics.
    json!({
        "content": [
            { "type": "text", "text": fallback }
        ],
        "structuredContent": {
            "chunks": chunks,
            "tokens_returned": search_result.tokens_returned,
            "truncated": search_result.truncated,
        }
    })
}

fn handle_list() -> Value {
    let db_dir = cache::cache_root().join("db");
    // M22: pair each docset with its unified install state so nowdocs_list
    // reports partial installs the same way list-installed / doctor / smoke do.
    let mut entries: Vec<(String, cache::InstalledDocsetState)> = Vec::new();

    if let Ok(read) = std::fs::read_dir(&db_dir) {
        for entry in read.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if let Some(stem) = name.strip_suffix(".lance") {
                        entries.push((stem.to_string(), cache::check_docset_state(stem)));
                    }
                }
            }
        }
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let text = if entries.is_empty() {
        "no docsets installed".to_string()
    } else {
        // S6: docset names are returned to the LLM; sanitize defensively even
        // though input::validate_docset already constrains the on-disk names.
        let joined = entries
            .iter()
            .map(|(name, state)| format!("{name} ({})", state.label()))
            .collect::<Vec<_>>()
            .join(", ");
        sanitize::sanitize_metadata(&joined)
    };

    json!({
        "content": [
            { "type": "text", "text": text }
        ]
    })
}

fn err_response(code: i64, message: &str) -> Value {
    json!({ "code": code, "message": message })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache;

    #[allow(non_snake_case)]
    #[test]
    fn tools_call_search_returns_isError_when_model_unavailable() {
        // N2: sentinel downcast — an EmbedderLoadError (anywhere in the
        // context chain) classifies as ModelUnavailable, no string matching.
        let err = anyhow::Error::new(retrieve::EmbedderLoadError("hf-hub download failed".into()))
            .context("embedder init");
        let te = classify_error(&err, "nextjs");
        assert!(
            matches!(te, ToolError::ModelUnavailable { .. }),
            "expected ModelUnavailable, got {te:?}"
        );
        let result = te.to_tool_result();
        assert_eq!(result["isError"], true);
        assert!(
            result["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("model"),
            "hint must mention the model, got: {:?}",
            result["content"][0]["text"]
        );
        // Internal detail must not leak into the client-facing text.
        assert!(
            !result["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("hf-hub download failed"),
            "raw error chain must not leak to the client"
        );
    }

    #[allow(non_snake_case)]
    #[test]
    fn tools_call_search_returns_isError_on_store_error() {
        // N2: sentinel downcast — a StoreError classifies as StoreCorrupt.
        let err = anyhow::Error::new(retrieve::StoreError {
            docset: "nextjs".into(),
            reason: "failed to open table".into(),
        });
        let te = classify_error(&err, "nextjs");
        assert!(
            matches!(te, ToolError::StoreCorrupt { .. }),
            "expected StoreCorrupt, got {te:?}"
        );
        let result = te.to_tool_result();
        assert_eq!(result["isError"], true);
        assert!(
            result["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("corrupt"),
            "hint must mention corruption, got: {:?}",
            result["content"][0]["text"]
        );
    }

    #[test]
    fn tools_call_search_classify_falls_back_to_internal() {
        // An unrelated error chain must not be misclassified as model/store.
        let err = anyhow::anyhow!("some unexpected condition");
        let te = classify_error(&err, "nextjs");
        assert!(matches!(te, ToolError::Internal { .. }));
        assert_eq!(te.to_tool_result()["isError"], true);
    }

    #[test]
    fn classify_error_downcasts_docset_not_installed() {
        // N2 defense-in-depth: the pre-flight manifest check in handle_search
        // is the primary not-installed detection, but if the manifest vanishes
        // between that check and retrieve::search's read, the DocsetNotInstalled
        // sentinel must still classify as NotInstalled (not StoreCorrupt).
        let err = anyhow::Error::new(retrieve::DocsetNotInstalled {
            docset: "nextjs".into(),
            reason: "failed to read manifest".into(),
        });
        let te = classify_error(&err, "nextjs");
        assert!(
            matches!(te, ToolError::NotInstalled { .. }),
            "expected NotInstalled, got {te:?}"
        );
        let result = te.to_tool_result();
        assert_eq!(result["isError"], true);
        assert!(
            result["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("not installed"),
            "hint must mention installation, got: {:?}",
            result["content"][0]["text"]
        );
    }

    #[test]
    fn test_handle_list_sanitizes_docset_names() {
        let dir = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("XDG_CACHE_HOME", dir.path()) };
        cache::ensure_layout().unwrap();

        // Plant a docset directory whose name embeds a zero-width injection char.
        // Real on-disk names are constrained by `validate_docset`, but
        // `handle_list` must still defend against hostile metadata reaching the
        // LLM, so its output must pass through `sanitize_metadata` (S6).
        let zw = char::from_u32(0x200B).unwrap();
        let db = cache::cache_root().join("db");
        std::fs::create_dir_all(db.join(format!("evil{zw}docset.lance"))).unwrap();

        let value = handle_list();
        let text = value["content"][0]["text"].as_str().unwrap();
        assert!(
            !text.contains(zw),
            "handle_list output must be sanitized, got: {text:?}"
        );
    }
}
