use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

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
