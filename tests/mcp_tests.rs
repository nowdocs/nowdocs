//! Integration test: spawn the `nowdocs serve` binary and talk MCP 2025-11-25
//! over stdio NDJSON.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

struct McpSession {
    stdin: std::process::ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
}

fn bin_path() -> String {
    // Cargo injects CARGO_BIN_EXE_nowdocs for integration tests in some
    // invocations; fall back to the manifest's target/debug path otherwise
    // (the bin is built explicitly before the test run).
    if let Ok(p) = std::env::var("CARGO_BIN_EXE_nowdocs") {
        return p;
    }
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    format!("{manifest}/target/debug/nowdocs")
}

fn spawn() -> (McpSession, std::process::Child) {
    let bin = bin_path();
    // Isolate the cache so ensure_layout writes into a tempdir, not the real
    // user cache. Leaked (not auto-cleaned) so it outlives the child process.
    let cache = tempfile::tempdir().unwrap();
    let cache_path = cache.path().to_path_buf();
    std::mem::forget(cache);

    let mut child = Command::new(&bin)
        .arg("serve")
        .env("XDG_CACHE_HOME", &cache_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn nowdocs serve");
    let stdin = child.stdin.take().unwrap();
    let stdout = BufReader::new(child.stdout.take().unwrap());
    let session = McpSession { stdin, stdout };
    (session, child)
}

impl McpSession {
    /// Send one NDJSON request and read back the single NDJSON response line
    /// matching its id.
    fn round_trip(&mut self, req: &serde_json::Value) -> serde_json::Value {
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
            // Skip notifications (no id) - we only expect responses with an id.
            if resp.get("id").is_some() {
                return resp;
            }
        }
    }
}

#[test]
fn initialize_returns_2025_11_25() {
    let (mut s, _child) = spawn();
    let req = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "0.0.0"}
        }
    });
    let resp = s.round_trip(&req);
    let result = resp.get("result").expect("initialize must return result");
    assert_eq!(result["protocolVersion"], nowdocs::mcp::PROTOCOL_VERSION);
    assert_eq!(result["capabilities"]["tools"]["listChanged"], false);
    assert_eq!(result["serverInfo"]["name"], "nowdocs");
}

#[test]
fn tools_list_exposes_search_and_list() {
    let (mut s, _child) = spawn();
    // initialize first (real clients always do).
    let _ = s.round_trip(&serde_json::json!({
        "jsonrpc":"2.0","id":1,"method":"initialize",
        "params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"t","version":"0"}}
    }));
    let resp = s.round_trip(&serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}));
    let tools = resp["result"]["tools"].as_array().expect("tools array");
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(
        names.contains(&"nowdocs_search"),
        "missing nowdocs_search: {:?}",
        names
    );
    assert!(
        names.contains(&"nowdocs_list"),
        "missing nowdocs_list: {:?}",
        names
    );

    let search = tools
        .iter()
        .find(|t| t["name"] == "nowdocs_search")
        .unwrap();
    assert!(
        search["inputSchema"].is_object(),
        "search needs inputSchema"
    );
    let required = search["inputSchema"]["required"].as_array().unwrap();
    let req_names: Vec<&str> = required.iter().map(|r| r.as_str().unwrap()).collect();
    assert!(req_names.contains(&"query"), "query must be required");
    assert!(req_names.contains(&"docset"), "docset must be required");
    assert_eq!(search["annotations"]["readOnlyHint"], true);
    assert_eq!(search["annotations"]["openWorldHint"], false);

    let list = tools.iter().find(|t| t["name"] == "nowdocs_list").unwrap();
    assert!(list["inputSchema"].is_object());
    assert_eq!(list["annotations"]["readOnlyHint"], true);
}

#[test]
fn tools_call_search_returns_structured_error_not_crash() {
    let (mut s, mut child) = spawn();
    let _ = s.round_trip(&serde_json::json!({
        "jsonrpc":"2.0","id":1,"method":"initialize",
        "params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"t","version":"0"}}
    }));
    let resp = s.round_trip(&serde_json::json!({
        "jsonrpc":"2.0","id":2,"method":"tools/call",
        "params":{"name":"nowdocs_search","arguments":{"query":"how to use middleware","docset":"nextjs"}}
    }));
    // Search is wired (Wave 4) but docset doesn't exist. A missing docset is a
    // business error: it must surface as a tool result with `isError: true`
    // (not a JSON-RPC error), never a crash / dropped connection.
    let result = resp.get("result").expect("expected a result envelope");
    assert_eq!(
        result["isError"].as_bool(),
        Some(true),
        "missing docset must be isError:true, got: {}",
        resp
    );
    assert!(
        resp.get("error").is_none(),
        "business errors must not be JSON-RPC errors, got: {}",
        resp
    );
    let text = result["content"][0]["text"].as_str().expect("hint text");
    assert!(
        text.contains("nextjs") && text.contains("install"),
        "hint must name the docset and the install command, got: {text:?}"
    );

    // Server must still be alive (not crashed).
    let alive = s.round_trip(&serde_json::json!({"jsonrpc":"2.0","id":3,"method":"tools/list"}));
    assert!(alive.get("result").is_some());
    // cleanup
    let _ = child.kill();
}

#[test]
fn tools_call_rejects_invalid_docset() {
    let (mut s, mut child) = spawn();
    let _ = s.round_trip(&serde_json::json!({
        "jsonrpc":"2.0","id":1,"method":"initialize",
        "params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"t","version":"0"}}
    }));
    let resp = s.round_trip(&serde_json::json!({
        "jsonrpc":"2.0","id":2,"method":"tools/call",
        "params":{"name":"nowdocs_search","arguments":{"query":"x","docset":"../etc"}}
    }));
    assert!(
        resp.get("error").is_some(),
        "invalid docset must error, got: {}",
        resp
    );
    let _ = child.kill();
}

// C2 (P2 repair): the tools/list handler must emit exactly the tool names
// from the single mcp::MCP_TOOL_NAMES source of truth.
#[test]
fn tools_list_names_equal_mcp_tool_names_constant() {
    assert_eq!(
        nowdocs::mcp::MCP_TOOL_NAMES,
        ["nowdocs_list", "nowdocs_search"],
        "MCP_TOOL_NAMES must be exactly the two read-only tools in lexical order"
    );
    let tools = nowdocs::mcp::handle_tools_list();
    let mut names: Vec<String> = tools
        .get("tools")
        .and_then(|t| t.as_array())
        .expect("tools array")
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect();
    names.sort();
    let expected: Vec<String> = nowdocs::mcp::MCP_TOOL_NAMES
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        names, expected,
        "tools/list emitted name set must equal MCP_TOOL_NAMES"
    );
}

// M4: malformed JSON sent to the MCP server must return JSON-RPC parse error
// code -32700 (not -32602 / ERR_INVALID_PARAMS).
#[test]
fn test_mcp_parse_error_returns_32700() {
    let cache = tempfile::tempdir().unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_nowdocs"))
        .arg("serve")
        .env("XDG_CACHE_HOME", cache.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn nowdocs serve");

    // Send malformed JSON (not valid JSON-RPC).
    let mut stdin = child.stdin.take().expect("child stdin");
    stdin
        .write_all(b"this is not valid json\n")
        .expect("write to child stdin");

    // Read the first response line; it must be a parse error with code -32700.
    let stdout = child.stdout.take().expect("child stdout");
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    reader.read_line(&mut line).expect("read response line");

    drop(stdin);
    let _ = child.kill();
    let _ = child.wait();

    let v: serde_json::Value = serde_json::from_str(&line).expect("response is JSON");
    assert_eq!(
        v["error"]["code"].as_i64(),
        Some(-32700),
        "malformed JSON must return parse error -32700, got: {line}"
    );
}

// M6: `tools/list` must declare `integer` type with bounds for max_tokens/top_k.
#[test]
fn test_tools_list_declares_integer_schema() {
    let (mut s, _child) = spawn();
    let _ = s.round_trip(&serde_json::json!({
        "jsonrpc":"2.0","id":1,"method":"initialize",
        "params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"t","version":"0"}}
    }));
    let resp = s.round_trip(&serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}));
    let tools = resp["result"]["tools"].as_array().expect("tools array");
    let search = tools
        .iter()
        .find(|t| t["name"] == "nowdocs_search")
        .expect("nowdocs_search tool");
    let props = &search["inputSchema"]["properties"];

    for field in ["max_tokens", "top_k"] {
        let prop = &props[field];
        assert_eq!(
            prop["type"], "integer",
            "{field} must be declared as integer, got: {prop:?}"
        );
        assert!(prop["minimum"].as_i64().is_some(), "{field} needs minimum");
        assert!(prop["maximum"].as_i64().is_some(), "{field} needs maximum");
        assert!(prop["default"].as_i64().is_some(), "{field} needs default");
    }

    // Sanity: the documented bounds for max_tokens.
    assert_eq!(props["max_tokens"]["minimum"].as_i64(), Some(1));
    assert_eq!(props["max_tokens"]["maximum"].as_i64(), Some(32768));
    assert_eq!(props["max_tokens"]["default"].as_i64(), Some(4096));
    assert_eq!(props["top_k"]["minimum"].as_i64(), Some(1));
    assert_eq!(props["top_k"]["maximum"].as_i64(), Some(50));
    assert_eq!(props["top_k"]["default"].as_i64(), Some(5));
    assert_eq!(
        props["top_k"]["description"].as_str(),
        Some("Number of top hybrid hits; each hit may include adjacent context chunks")
    );
}

// M7: a request line larger than 1 MiB must be rejected with JSON-RPC parse
// error -32700 and the server must not panic / OOM.
#[test]
fn test_mcp_oversized_line_returns_32700() {
    let cache = tempfile::tempdir().unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_nowdocs"))
        .arg("serve")
        .env("XDG_CACHE_HOME", cache.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn nowdocs serve");

    // Build a ~2 MiB line with no internal newline, then terminate with one.
    let mut payload = String::with_capacity(2 * 1024 * 1024);
    payload.push_str("{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"");
    while payload.len() < 2 * 1024 * 1024 {
        payload.push('a');
    }
    payload.push_str("\"}}}");
    let mut line_bytes = payload.into_bytes();
    line_bytes.push(b'\n');

    let mut stdin = child.stdin.take().expect("child stdin");
    stdin.write_all(&line_bytes).expect("write oversized line");
    stdin.flush().expect("flush");

    let stdout = child.stdout.take().expect("child stdout");
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    reader.read_line(&mut line).expect("read response line");

    drop(stdin);
    let _ = child.kill();
    let _ = child.wait();

    let v: serde_json::Value = serde_json::from_str(&line).expect("response is JSON");
    assert_eq!(
        v.get("error")
            .and_then(|e| e.get("code"))
            .and_then(|c| c.as_i64()),
        Some(-32700),
        "oversized line must return parse error -32700, got: {line}"
    );
}

// ---------------------------------------------------------------------------
// Cached update reminder on serve (unified update service)
//
// serve is cache-only: it may claim a fresh unnotified reminder and write it
// exactly once to stderr. It must never initiate a network request, stdout
// must remain protocol-clean NDJSON, and two sequential serve processes must
// not both claim the same reminder.
// ---------------------------------------------------------------------------

/// Write a seeded update-cache.json into `<cache_home>/nowdocs/` with a fresh
/// unnotified newer version, so serve claims the reminder without fetching.
fn seed_update_cache(cache_home: &std::path::Path) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let cache_json = serde_json::json!({
        "schema_version": 1,
        "running_version": env!("CARGO_PKG_VERSION"),
        "last_attempt_secs": now,
        "last_success_secs": now,
        "latest_version": "99.0.0",
        "notified_version": null,
    });
    let dir = cache_home.join("nowdocs");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("update-cache.json"), cache_json.to_string()).unwrap();
}

/// Spawn `nowdocs serve` with an unreachable proxy and a seeded cache, capturing
/// stderr. Returns the child handle so the caller can interact with it.
fn spawn_with_cache(cache_path: &std::path::Path) -> (McpSession, std::process::Child) {
    let bin = bin_path();
    let mut child = Command::new(&bin)
        .arg("serve")
        .env("XDG_CACHE_HOME", cache_path)
        // Unreachable proxy so any accidental network attempt fails fast.
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

/// serve with a seeded cache must: (1) print exactly one reminder to stderr,
/// (2) keep stdout protocol-clean, (3) never make a network request.
#[test]
fn cached_update_reminder() {
    let cache = tempfile::tempdir().unwrap();
    let cache_path = cache.path().to_path_buf();
    seed_update_cache(&cache_path);

    let (mut s, child) = spawn_with_cache(&cache_path);

    // Send initialize - the response must be clean JSON-RPC on stdout.
    let resp = s.round_trip(&serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "0.0.0"}
        }
    }));
    let result = resp.get("result").expect("initialize must return result");
    assert_eq!(result["protocolVersion"], nowdocs::mcp::PROTOCOL_VERSION);
    assert_eq!(result["serverInfo"]["name"], "nowdocs");

    // Do a tools/list to be sure stdout is fully protocol-clean.
    let resp2 = s.round_trip(&serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}));
    assert!(
        resp2.get("result").is_some(),
        "tools/list must return result"
    );

    // Close stdin so the server exits, then read stderr.
    drop(s.stdin);
    let output = child.wait_with_output().expect("wait for child");

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    // stdout must be valid NDJSON (every non-empty line is a JSON object with
    // jsonrpc field). No reminder text on stdout.
    for line in stdout_str.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let v: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("stdout line not JSON: {line:?} ({e})"));
        assert!(
            v.get("jsonrpc").is_some(),
            "stdout must be JSON-RPC, got: {line}"
        );
        assert!(
            !line.contains("99.0.0"),
            "stdout must not contain the update reminder, got: {line}"
        );
    }

    // stderr must contain exactly one reminder with the version.
    assert!(
        stderr_str.contains("99.0.0"),
        "stderr must contain the update reminder, got: {stderr_str}"
    );
    let count = stderr_str.matches("99.0.0").count();
    assert_eq!(
        count, 1,
        "stderr must contain the reminder exactly once, got {count} times: {stderr_str}"
    );
}

/// Two sequential serve processes must not both claim the same reminder: the
/// first claims it (marks notified), the second sees it as already notified.
#[test]
fn cached_update_reminder_no_duplicate_claim() {
    let cache = tempfile::tempdir().unwrap();
    let cache_path = cache.path().to_path_buf();
    seed_update_cache(&cache_path);

    // First serve process: claims the reminder.
    let (mut s1, child1) = spawn_with_cache(&cache_path);
    let _ = s1.round_trip(&serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "t", "version": "0"}}
    }));
    drop(s1.stdin);
    let out1 = child1.wait_with_output().expect("wait child1");
    let stderr1 = String::from_utf8_lossy(&out1.stderr);
    assert!(
        stderr1.contains("99.0.0"),
        "first serve must claim the reminder, got stderr: {stderr1}"
    );

    // Second serve process: must NOT claim (already notified).
    let (mut s2, child2) = spawn_with_cache(&cache_path);
    let _ = s2.round_trip(&serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "t", "version": "0"}}
    }));
    drop(s2.stdin);
    let out2 = child2.wait_with_output().expect("wait child2");
    let stderr2 = String::from_utf8_lossy(&out2.stderr);
    assert!(
        !stderr2.contains("99.0.0"),
        "second serve must not re-claim the reminder, got stderr: {stderr2}"
    );
}

// ---------------------------------------------------------------------------
// C8: the MCP tool surface stays read-only. `verify` is a CLI-only command and
// must never appear as an MCP tool. tools/list continues to advertise exactly
// the two read-only tools.
// ---------------------------------------------------------------------------

#[test]
fn tools_list_does_not_advertise_verify_as_mcp_tool() {
    let (mut s, _child) = spawn();
    let _ = s.round_trip(&serde_json::json!({
        "jsonrpc":"2.0","id":1,"method":"initialize",
        "params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"t","version":"0"}}
    }));
    let resp = s.round_trip(&serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}));
    let tools = resp["result"]["tools"].as_array().expect("tools array");
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(
        !names.contains(&"verify"),
        "verify must not be an MCP tool, got: {names:?}"
    );
    assert_eq!(
        nowdocs::mcp::MCP_TOOL_NAMES.len(),
        2,
        "exactly two read-only MCP tools"
    );
    assert!(
        !nowdocs::mcp::MCP_TOOL_NAMES.contains(&"verify"),
        "MCP_TOOL_NAMES must not contain verify"
    );
}
