# Getting Started with nowdocs

This guide is the shortest supported path from a fresh checkout or install to a working MCP doc server.

## 1. Install or build

Current source install:

```bash
cargo install --git https://github.com/nowdocs/nowdocs
```

From a local checkout:

```bash
cargo build --release
```

Requirements:

- Rust toolchain compatible with the dependency graph in `Cargo.lock`.
- `protoc` plus protobuf well-known type include files, required by LanceDB/prost build dependencies.

## 2. Run diagnostics first

```bash
nowdocs doctor
nowdocs doctor --json
```

Use `--json` when an agent or CI needs machine-readable check output.

## 3. Install or ingest a docset

Registry install, when the docset exists in the curated registry:

```bash
nowdocs install <docset>
```

Local Markdown ingest:

```bash
nowdocs ingest ./my-docs my-docset --license MIT --source-url https://github.com/org/repo
```

For CC-BY-4.0 content, include attribution:

```bash
nowdocs ingest ./my-docs my-docset \
  --license CC-BY-4.0 \
  --attribution "Docs by Example Authors" \
  --source-url https://github.com/org/repo
```

## 4. Verify retrieval with smoke

Run a real retrieval smoke test before wiring an MCP client:

```bash
nowdocs smoke my-docset "installation configuration example"
```

JSON output for agents/CI:

```bash
nowdocs smoke my-docset "installation configuration example" --json --top-k 3
```

If smoke fails because model files are missing or corrupt, run:

```bash
nowdocs doctor --model
```

## 5. Start the MCP server

```bash
nowdocs serve
```

`serve` uses stdio NDJSON. It does not bind a host or port.

## 6. Configure an MCP client

Generic MCP JSON:

```json
{
  "mcpServers": {
    "nowdocs": {
      "command": "nowdocs",
      "args": ["serve"]
    }
  }
}
```

Client-specific snippets live in [`MCP_CLIENTS.md`](MCP_CLIENTS.md).

## 7. Useful recovery commands

Inspect cache state:

```bash
nowdocs cache status
nowdocs cache status --json
```

Clean stale staging directories only:

```bash
nowdocs cache clean-staging --older-than 1h
```

Run safe repair through doctor:

```bash
nowdocs doctor --repair
```

`doctor --repair` and `cache clean-staging` must not delete active docsets.
