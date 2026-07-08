# nowdocs Troubleshooting

Use this guide when install, ingest, search, cache, or MCP setup fails.

## 1. Model download or model cache failure

Symptoms:

- `nowdocs smoke` fails during embedding.
- Error mentions model download, missing weights, tokenizer, SHA mismatch, or HuggingFace access.

Checks:

```bash
nowdocs doctor --model
```

Fixes:

- Retry on a network that can reach HuggingFace for the first model download.
- If a SHA mismatch is reported, remove only the affected model cache directory and retry.
- Do not edit manifest embedder fields manually; model metadata is pinned for safety.

## 2. Docset not found

Symptoms:

- `nowdocs smoke <docset>` reports the docset is missing.
- MCP search returns an error for the requested docset.

Checks:

```bash
nowdocs list-installed
nowdocs doctor --docset <docset>
```

Fixes:

```bash
nowdocs install <docset>
# or
nowdocs ingest ./my-docs <docset> --license MIT --source-url https://github.com/org/repo
```

## 3. Corrupt manifest or missing store

Symptoms:

- `list-installed` shows a broken status.
- `doctor --docset` reports manifest/store mismatch.

Checks:

```bash
nowdocs doctor --docset <docset>
```

Fixes:

- Reinstall registry docsets with `nowdocs install <docset>` or `nowdocs update <docset>`.
- Re-run local ingest for local docsets.
- Use `nowdocs uninstall <docset>` only when you intend to remove the active docset.

## 4. Stale staging directories after interrupted install/update

Symptoms:

- `doctor` warns about staging directories.
- `cache status` shows non-zero staging count or staging bytes.

Checks:

```bash
nowdocs cache status
nowdocs doctor
```

Fixes:

```bash
nowdocs cache clean-staging --older-than 1h
nowdocs doctor --repair
```

These commands remove only nowdocs-owned staging directories and must not remove active `.lance` stores, manifests, or model files.

## 5. MCP tools not visible in client

Symptoms:

- Client does not show `nowdocs_search` or `nowdocs_list`.
- Client cannot initialize the server.

Checks:

```bash
nowdocs doctor --mcp
```

Fixes:

- Confirm the client command is `nowdocs` and args are `["serve"]`.
- Confirm no `--host` or `--port` flags are configured; nowdocs serves over stdio only.
- Restart the MCP client after changing config.
- See [`MCP_CLIENTS.md`](MCP_CLIENTS.md) for snippets.

## 6. Search returns no useful results

Symptoms:

- `smoke` returns zero results or irrelevant hits.
- Agent receives context but not the expected API documentation.

Checks:

```bash
nowdocs smoke <docset> "specific API or error message" --top-k 5
nowdocs doctor --docset <docset>
```

Fixes:

- Use a more specific query with API names, file names, or exact error strings.
- Re-ingest if the source docs changed.
- For release validation, run the real-docset eval gate before publishing.

## 7. Source build fails around Rust version or protoc

Symptoms:

- Build says a dependency requires a newer Rust compiler.
- Build fails with missing `protoc` or `google/protobuf/empty.proto`.

Fixes:

- Use the Rust version required by the current `Cargo.lock` dependency graph.
- Install `protoc` and protobuf well-known type include files.
- In constrained environments, set `PROTOC=/path/to/protoc` if the binary is not on `PATH`.

## 8. OS cache paths

nowdocs uses the OS cache directory plus `nowdocs`:

- Linux: usually `~/.cache/nowdocs` or `$XDG_CACHE_HOME/nowdocs`.
- macOS: the platform cache directory returned by the OS.
- Windows: the platform cache directory returned by the OS.

Useful commands:

```bash
nowdocs cache status
nowdocs list-installed
```

Do not manually delete active cache paths unless a maintainer specifically asks you to; prefer `uninstall`, `doctor`, and `cache clean-staging`.
