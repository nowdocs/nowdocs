//! C8 end-to-end agent flow: spawn the real `nowdocs serve` binary and exercise
//! the documented MCP request order over stdio NDJSON:
//!
//! `initialize -> tools/list -> tools/call(nowdocs_list) ->
//! tools/call(nowdocs_search with explicit docset)`.
//!
//! The harness extends the existing spawned-binary pattern from
//! `tests/mcp_tests.rs` rather than replacing it. The default hermetic harness
//! asserts the documented structured no-docset result for search (no real
//! embedder model is loaded; no network is used).

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

struct McpSession {
    stdin: std::process::ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
}

fn bin_path() -> String {
    if let Ok(p) = std::env::var("CARGO_BIN_EXE_nowdocs") {
        return p;
    }
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    format!("{manifest}/target/debug/nowdocs")
}

/// Spawn `nowdocs serve` with an isolated cache and deliberately unreachable
/// proxy variables so any accidental network attempt fails fast.
fn spawn() -> (McpSession, std::process::Child) {
    let bin = bin_path();
    let cache = tempfile::tempdir().unwrap();
    let cache_path = cache.path().to_path_buf();
    std::mem::forget(cache);

    let mut child = Command::new(&bin)
        .arg("serve")
        .env("XDG_CACHE_HOME", &cache_path)
        .env("HOME", &cache_path)
        .env("XDG_CONFIG_HOME", &cache_path)
        .env("XDG_DATA_HOME", &cache_path)
        .env("http_proxy", "http://127.0.0.1:9")
        .env("https_proxy", "http://127.0.0.1:9")
        .env("HTTP_PROXY", "http://127.0.0.1:9")
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .env("ALL_PROXY", "http://127.0.0.1:9")
        .env("no_proxy", "")
        .env("NO_PROXY", "")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn nowdocs serve");
    let stdin = child.stdin.take().unwrap();
    let stdout = BufReader::new(child.stdout.take().unwrap());
    let session = McpSession { stdin, stdout };
    (session, child)
}

impl McpSession {
    /// Send one NDJSON request and read back the single NDJSON response line
    /// whose id exactly equals the request's id. This catches id-mismatch bugs.
    fn round_trip(&mut self, req: &serde_json::Value) -> serde_json::Value {
        let req_id = req.get("id").cloned();
        let mut line = serde_json::to_string(req).unwrap();
        line.push('\n');
        self.stdin.write_all(line.as_bytes()).unwrap();
        self.stdin.flush().unwrap();

        let mut buf = String::new();
        loop {
            buf.clear();
            let n = self.stdout.read_line(&mut buf).expect("read response");
            assert!(n > 0, "server closed stdout before responding to {:?}", req);
            let trimmed = buf.trim();
            if trimmed.is_empty() {
                continue;
            }
            let resp: serde_json::Value = serde_json::from_str(trimmed)
                .unwrap_or_else(|e| panic!("non-JSON response: {trimmed:?} ({e})"));
            // Skip notifications (no id). For requests, assert the response id
            // exactly equals the request id -- never accept any id-bearing
            // response.
            if let Some(rid) = &req_id {
                if let Some(rsp_id) = resp.get("id") {
                    assert_eq!(
                        rsp_id, rid,
                        "response id must exactly equal request id; got {rsp_id} for request {rid}"
                    );
                    return resp;
                }
                // id-less response to an id-bearing request: skip (shouldn't
                // happen, but be conservative).
            }
        }
    }

    /// Send one NDJSON request and return the raw response line (including the
    /// trailing newline) whose id matches. Used to capture exact wire bytes.
    fn round_trip_raw(&mut self, req: &serde_json::Value) -> String {
        let req_id = req.get("id").cloned();
        let mut line = serde_json::to_string(req).unwrap();
        line.push('\n');
        self.stdin.write_all(line.as_bytes()).unwrap();
        self.stdin.flush().unwrap();

        let mut buf = String::new();
        loop {
            buf.clear();
            let n = self.stdout.read_line(&mut buf).expect("read response");
            assert!(n > 0, "server closed stdout before responding to {:?}", req);
            let trimmed = buf.trim();
            if trimmed.is_empty() {
                continue;
            }
            let resp: serde_json::Value = serde_json::from_str(trimmed)
                .unwrap_or_else(|e| panic!("non-JSON response: {trimmed:?} ({e})"));
            if let Some(rid) = &req_id {
                if let Some(rsp_id) = resp.get("id") {
                    assert_eq!(
                        rsp_id, rid,
                        "response id must exactly equal request id; got {rsp_id} for request {rid}"
                    );
                    return buf.clone();
                }
            }
        }
    }
}

/// The full agent-flow request order over the real binary: initialize, then
/// tools/list, then nowdocs_list, then nowdocs_search with an explicit docset.
/// Every response must be protocol-clean JSON-RPC, and the server must stay
/// alive through the whole sequence.
#[test]
fn agent_flow_initialize_list_then_search_in_order() {
    let (mut s, mut child) = spawn();

    // 1. initialize
    let init = s.round_trip(&serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "agent-flow-e2e", "version": "0.0.0"}
        }
    }));
    let result = init.get("result").expect("initialize must return result");
    assert_eq!(
        result["protocolVersion"],
        nowdocs::mcp::PROTOCOL_VERSION,
        "protocol must be 2025-11-25"
    );
    assert_eq!(result["serverInfo"]["name"], "nowdocs");

    // 2. tools/list - exactly the two read-only tools, no verify tool.
    let list = s.round_trip(&serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}));
    let tools = list["result"]["tools"]
        .as_array()
        .expect("tools/list returns a tools array");
    let mut names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    names.sort();
    assert_eq!(
        names,
        ["nowdocs_list", "nowdocs_search"],
        "exactly the two read-only MCP tools must be advertised; verify is not an MCP tool"
    );

    // 3. tools/call nowdocs_list - no docsets installed in the isolated cache.
    let call_list = s.round_trip(&serde_json::json!({
        "jsonrpc":"2.0","id":3,"method":"tools/call",
        "params":{"name":"nowdocs_list","arguments":{}}
    }));
    let list_result = call_list
        .get("result")
        .expect("nowdocs_list must return a result envelope");
    let list_text = list_result["content"][0]["text"]
        .as_str()
        .expect("nowdocs_list text");
    assert!(
        list_text.contains("no docsets installed"),
        "isolated cache must report no docsets installed, got: {list_text:?}"
    );

    // 4. tools/call nowdocs_search with an explicit docset that is not installed.
    //    This must surface a structured business result (isError: true) with an
    //    install hint, never a JSON-RPC error or a crash.
    let call_search = s.round_trip(&serde_json::json!({
        "jsonrpc":"2.0","id":4,"method":"tools/call",
        "params":{"name":"nowdocs_search","arguments":{"query":"how to use middleware","docset":"nextjs"}}
    }));
    let search_result = call_search
        .get("result")
        .expect("search must return a result envelope, not a JSON-RPC error");
    assert_eq!(
        search_result["isError"].as_bool(),
        Some(true),
        "missing docset must be isError:true, got: {}",
        call_search
    );
    assert!(
        call_search.get("error").is_none(),
        "business errors must not be JSON-RPC errors, got: {}",
        call_search
    );
    let hint = search_result["content"][0]["text"]
        .as_str()
        .expect("search hint text");
    assert!(
        hint.contains("nextjs") && hint.contains("install"),
        "search hint must name the docset and the install command, got: {hint:?}"
    );

    // Server must still be alive after the full sequence.
    let alive = s.round_trip(&serde_json::json!({"jsonrpc":"2.0","id":5,"method":"tools/list"}));
    assert!(alive.get("result").is_some(), "server must stay alive");

    drop(s.stdin);
    let _ = child.kill();
    let _ = child.wait();
}

/// Capture the actual response lines for a known request sequence and assert
/// the exact count, ids, and order. This cannot pass on empty stdout: it
/// requires at least one captured response line per request, and the ids must
/// appear in the exact request order (1, 2, 3).
#[test]
fn agent_flow_stdout_is_protocol_clean_ndjson() {
    let (mut s, child) = spawn();

    // Capture the raw response lines (each is one JSON-RPC document + newline).
    let r1 = s.round_trip_raw(&serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "ndjson-clean", "version": "0.0.0"}
        }
    }));
    let r2 = s.round_trip_raw(&serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}));
    let r3 = s.round_trip_raw(&serde_json::json!({
        "jsonrpc":"2.0","id":3,"method":"tools/call",
        "params":{"name":"nowdocs_list","arguments":{}}
    }));

    drop(s.stdin);
    let output = child.wait_with_output().expect("wait for child");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // The three captured responses must each be a single non-empty JSON line.
    let captured = [r1, r2, r3];
    let mut parsed: Vec<serde_json::Value> = Vec::new();
    for (i, raw) in captured.iter().enumerate() {
        let trimmed = raw.trim();
        assert!(
            !trimmed.is_empty(),
            "captured response {} must be non-empty (cannot pass on empty stdout)",
            i + 1
        );
        let v: serde_json::Value = serde_json::from_str(trimmed)
            .unwrap_or_else(|e| panic!("captured response {} not JSON: {trimmed:?} ({e})", i + 1));
        assert!(
            v.get("jsonrpc").is_some(),
            "captured response {} must be JSON-RPC, got: {trimmed}",
            i + 1
        );
        parsed.push(v);
    }

    // Exact id count and order: ids must be 1, 2, 3 in sequence.
    assert_eq!(
        parsed.len(),
        3,
        "exactly three responses must be captured, got {}",
        parsed.len()
    );
    let ids: Vec<serde_json::Value> = parsed.iter().map(|v| v["id"].clone()).collect();
    assert_eq!(
        ids,
        [
            serde_json::json!(1),
            serde_json::json!(2),
            serde_json::json!(3)
        ],
        "response ids must be exactly [1, 2, 3] in order, got {ids:?}"
    );

    // No second JSON document on any captured line: each raw line must parse as
    // exactly one value (serde_json::from_str already rejects trailing data).
    // Also assert the full stdout (including any tail after stdin close) is
    // protocol-clean: every non-empty line is a JSON-RPC object.
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let v: serde_json::Value = serde_json::from_str(trimmed)
            .unwrap_or_else(|e| panic!("stdout tail line not JSON: {line:?} ({e})"));
        assert!(
            v.get("jsonrpc").is_some(),
            "every stdout line must be JSON-RPC, got: {line}"
        );
    }
}
