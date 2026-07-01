//! MCP tool handlers (nowdocs_search / nowdocs_list).

use serde_json::{json, Value};

use crate::{cache, input, retrieve, sanitize};

/// JSON-RPC error codes.
const ERR_INVALID_PARAMS: i64 = -32602;
const ERR_METHOD_NOT_FOUND: i64 = -32601;

/// Dispatch an MCP tool call by name.
///
/// Returns a JSON-RPC `result` on success, or an `error` object on failure.
pub fn handle_call(name: &str, args: Value) -> Value {
    match name {
        "nowdocs_search" => handle_search(args),
        "nowdocs_list" => handle_list(),
        other => err_response(ERR_METHOD_NOT_FOUND, &format!("unknown tool: {other}")),
    }
}

fn handle_search(args: Value) -> Value {
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

    // Run retrieval pipeline.
    let search_result = match retrieve::search(docset, query, max_tokens, top_k) {
        Ok(r) => r,
        Err(e) => return err_response(ERR_INVALID_PARAMS, &format!("search failed: {e}")),
    };

    // Sanitize each chunk.
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
    let mut docsets: Vec<String> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&db_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if let Some(stem) = name.strip_suffix(".lance") {
                        docsets.push(stem.to_string());
                    }
                }
            }
        }
    }

    docsets.sort();

    let text = if docsets.is_empty() {
        "no docsets installed".to_string()
    } else {
        docsets.join(", ")
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
