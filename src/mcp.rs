//! MCP 2025-11-25 server over stdio (NDJSON: one JSON-RPC message per `\n`).
//!
//! Wave 1 skeleton: handles `initialize`, `tools/list`, and `tools/call`.
//! The search/list tool handlers are stubbed with JSON-RPC error `-32601`
//! ("tool not yet implemented") — Wave 4 Task 4b/4c wires real retrieval.
//!
//! Inputs cross the trust boundary at `tools/call`, so `nowdocs_search`
//! arguments are validated via `input::validate_*` before anything else;
//! invalid input is rejected with a structured error (never a crash).

use std::io::{self, BufRead, Write};

use serde_json::{json, Value};

use crate::{cache, tools};

const PROTOCOL_VERSION: &str = "2025-11-25";
const SERVER_NAME: &str = "nowdocs";

/// JSON-RPC error codes.
const ERR_METHOD_NOT_FOUND: i64 = -32601;
const ERR_INVALID_PARAMS: i64 = -32602;
/// Parse error (JSON-RPC 2.0 §5.1): malformed JSON, id unknown.
const ERR_PARSE_ERROR: i64 = -32700;

pub fn run_loop() -> io::Result<()> {
    // Fail-closed: refuse to serve if the cache layout is wrong (D15).
    if let Err(e) = cache::ensure_layout() {
        eprintln!("nowdocs: cache layout error: {e}");
        return Err(io::Error::other(e.to_string()));
    }

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let msg: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                // Parse error — no id known, use null per JSON-RPC spec.
                let _ = write_response(
                    &mut out,
                    &Value::Null,
                    &err_response(ERR_PARSE_ERROR, &format!("parse error: {e}")),
                );
                continue;
            }
        };

        // Notifications (no id) get no response.
        let id = msg.get("id").cloned();
        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");

        let result: Option<Value> = match method {
            "initialize" => Some(handle_initialize()),
            "tools/list" => Some(handle_tools_list()),
            "tools/call" => Some(handle_tools_call(msg.get("params"))),
            // Unknown request method.
            _ if id.is_some() => {
                let _ = write_response(
                    &mut out,
                    id.as_ref().unwrap(),
                    &err_response(ERR_METHOD_NOT_FOUND, &format!("method not found: {method}")),
                );
                continue;
            }
            // Unknown notification — ignore.
            _ => continue,
        };

        if let Some(id) = id {
            let _ = write_response(&mut out, &id, &result.unwrap());
        }
    }

    Ok(())
}

pub fn handle_initialize() -> Value {
    json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": {
            "tools": { "listChanged": false }
        },
        "serverInfo": {
            "name": SERVER_NAME,
            "version": env!("CARGO_PKG_VERSION")
        }
    })
}

pub fn handle_tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "nowdocs_search",
                "description": "Search an installed docset for up-to-date third-party library documentation.",
                "inputSchema": {
                    "type": "object",
                    "required": ["query", "docset"],
                    "properties": {
                        "query": { "type": "string" },
                        "docset": { "type": "string" },
                        "max_tokens": { "type": "number" },
                        "top_k": { "type": "number" }
                    }
                },
                "annotations": { "readOnlyHint": true, "openWorldHint": false }
            },
            {
                "name": "nowdocs_list",
                "description": "List installed docsets.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                },
                "annotations": { "readOnlyHint": true, "openWorldHint": false }
            }
        ]
    })
}

fn handle_tools_call(params: Option<&Value>) -> Value {
    let params = match params {
        Some(p) => p,
        None => return err_response(ERR_INVALID_PARAMS, "missing params"),
    };
    let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
    let args = params.get("arguments").cloned().unwrap_or(json!({}));

    // Delegate to tools module.
    tools::handle_call(name, args)
}

fn err_response(code: i64, message: &str) -> Value {
    json!({ "code": code, "message": message })
}

fn write_response(out: &mut impl Write, id: &Value, body: &Value) -> io::Result<()> {
    let resp = if body.get("code").and_then(|c| c.as_i64()).is_some() {
        json!({ "jsonrpc": "2.0", "id": id, "error": body })
    } else {
        json!({ "jsonrpc": "2.0", "id": id, "result": body })
    };
    serde_json::to_writer(&mut *out, &resp)?;
    out.write_all(b"\n")?;
    out.flush()
}
