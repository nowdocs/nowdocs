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

use crate::{cache, embedder, tools};

const PROTOCOL_VERSION: &str = "2025-11-25";
const SERVER_NAME: &str = "nowdocs";

/// JSON-RPC error codes.
const ERR_METHOD_NOT_FOUND: i64 = -32601;
const ERR_INVALID_PARAMS: i64 = -32602;
/// Parse error (JSON-RPC 2.0 §5.1): malformed JSON, id unknown.
const ERR_PARSE_ERROR: i64 = -32700;

/// Maximum size of a single NDJSON request line (1 MiB). A client that sends a
/// newline-less line larger than this would otherwise OOM the process via
/// `BufRead::lines()`. Lines exceeding the cap are rejected with `-32700` and
/// discarded (M7).
const MAX_LINE_BYTES: usize = 1024 * 1024;

pub fn run_loop() -> io::Result<()> {
    // Fail-closed: refuse to serve if the cache layout is wrong (D15).
    if let Err(e) = cache::ensure_layout() {
        eprintln!("nowdocs: cache layout error: {e}");
        return Err(io::Error::other(e.to_string()));
    }

    // N3: warm the default embedder before the read loop so the first search is
    // fast. Best-effort + offline-safe: a cold cache (model not yet downloaded)
    // skips instantly and the first search loads on demand instead.
    embedder::preload_default_embedder();

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut reader = stdin.lock();
    let mut buf: Vec<u8> = Vec::with_capacity(1024);

    loop {
        buf.clear();
        // Bounded read: stop at `\n`, EOF, or MAX_LINE_BYTES+1 bytes. A
        // newline-less giant line would otherwise OOM the process via
        // `BufRead::lines()`, so we cap the read and reject the over-long line.
        let n = match read_line_capped(&mut reader, &mut buf, MAX_LINE_BYTES + 1) {
            Ok(n) => n,
            Err(_) => break,
        };
        if n == 0 {
            break; // EOF
        }

        if buf.len() > MAX_LINE_BYTES {
            // Oversized logical line. Drain the rest of this line so we don't
            // emit one error per buffer chunk, then reject with -32700.
            let ends_with_newline = buf.last() == Some(&b'\n');
            if !ends_with_newline {
                drain_rest_of_line(&mut reader)?;
            }
            let _ = write_response(
                &mut out,
                &Value::Null,
                &err_response(
                    ERR_PARSE_ERROR,
                    "request line exceeds 1 MiB maximum; discarded",
                ),
            );
            continue;
        }

        let trimmed = match std::str::from_utf8(&buf) {
            Ok(s) => s.trim(),
            Err(_) => {
                let _ = write_response(
                    &mut out,
                    &Value::Null,
                    &err_response(ERR_PARSE_ERROR, "invalid UTF-8 in request line"),
                );
                continue;
            }
        };
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

/// Read bytes from `reader` into `buf` until `\n`, EOF, or `cap` bytes, without
/// growing `buf` past `cap`. The remainder of an over-long line is left in
/// `reader`'s internal buffer (via `consume`) so a subsequent call continues the
/// same logical line.
fn read_line_capped(reader: &mut impl BufRead, buf: &mut Vec<u8>, cap: usize) -> io::Result<usize> {
    let mut total = 0usize;
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return Ok(total); // EOF
        }
        if let Some(pos) = available.iter().position(|&b| b == b'\n') {
            let take = (pos + 1).min(cap - total);
            buf.extend_from_slice(&available[..take]);
            reader.consume(take);
            total += take;
            return Ok(total);
        }
        let remaining = cap - total;
        if remaining == 0 {
            return Ok(total);
        }
        let take = available.len().min(remaining);
        buf.extend_from_slice(&available[..take]);
        reader.consume(take);
        total += take;
        if total >= cap {
            return Ok(total);
        }
    }
}

/// Drain the remainder of an oversized line (left buffered by `read_line_capped`)
/// up to the next `\n` or EOF, bounded per chunk so it cannot OOM the drain.
fn drain_rest_of_line(reader: &mut impl BufRead) -> io::Result<()> {
    let mut sink: Vec<u8> = Vec::with_capacity(1024);
    loop {
        sink.clear();
        let n = read_line_capped(reader, &mut sink, MAX_LINE_BYTES + 1)?;
        if n == 0 || sink.last() == Some(&b'\n') {
            return Ok(());
        }
    }
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
                        "max_tokens": {
                            "type": "integer",
                            "minimum": 1,
                            "maximum": 32768,
                            "default": 4096,
                            "description": "Maximum tokens to return"
                        },
                        "top_k": {
                            "type": "integer",
                            "minimum": 1,
                            "maximum": 50,
                            "default": 5,
                            "description": "Number of top hybrid hits; each hit may include adjacent context chunks"
                        }
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
