# nowdocs Troubleshooting

Use this guide when installation, ingest, search, cache, agent setup, verification, reranking, or MCP setup fails.

## 1. Model download or model cache failure

Symptoms:

- `nowdocs smoke` fails during embedding.
- An error mentions model download, missing weights, tokenizer, SHA mismatch, or Hugging Face access.

Check:

```bash
nowdocs doctor --model
```

Fixes:

- Retry on a network that can reach Hugging Face for the first model download.
- If a SHA mismatch is reported, remove only the affected model cache directory and retry.
- Do not edit manifest embedder fields manually; model metadata is pinned for safety.
- Remember that `nowdocs verify` is cached-only and never downloads the model.

## 2. Docset not found

Symptoms:

- `nowdocs smoke <docset>` reports that the docset is missing.
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
- `doctor --docset` reports a manifest or store mismatch.

Check:

```bash
nowdocs doctor --docset <docset>
```

Fixes:

- Reinstall registry docsets with `nowdocs install <docset>` or `nowdocs update <docset>`.
- Re-run local ingest for local docsets.
- Use `nowdocs uninstall <docset>` only when you intend to remove the active docset.

## 4. Stale staging directories after interrupted install or update

Symptoms:

- `doctor` warns about staging directories.
- `cache status` shows a non-zero staging count or staging bytes.

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

## 5. MCP tools not visible in the client

Symptoms:

- The client does not show `nowdocs_search` or `nowdocs_list`.
- The client cannot initialize the server.

Check:

```bash
nowdocs doctor --mcp
```

Fixes:

- Confirm that the client command points to the expected nowdocs binary and that its argument is `serve`.
- Confirm that no `--host` or `--port` flags are configured; nowdocs serves over stdio only.
- For Codex CLI, inspect the registration with `codex mcp get nowdocs --json`.
- For Claude Code, inspect it with `claude mcp get nowdocs`.
- Restart or reload the MCP client after changing its registration.
- See [MCP Clients](MCP_CLIENTS.md) for exact client boundaries and commands.

## 6. Native Cohere reranking does not start or falls back

Symptoms:

- `nowdocs serve` or `nowdocs smoke` reports a reranker configuration error.
- A desktop MCP client appears to use local ranking after Cohere was enabled.

Checks and fixes:

- Confirm that the server process receives `NOWDOCS_RERANK_PROVIDER=cohere`, `NOWDOCS_RERANK_MODEL`, and `COHERE_API_KEY`.
- Keep `NOWDOCS_RERANK_TIMEOUT_MS` between 100 and 10000 when it is set.
- Confirm that the model is available to the configured Cohere account.
- A failed Cohere request deliberately uses the unchanged local ranking for that search. Check account access, network connectivity, timeout, and rate limits.
- A failed request may already have transmitted its input; fallback does not reverse that transmission.
- See [Optional Reranking](RERANKING.md) for configuration, data-transfer, verification, and disable instructions.

## 7. Search returns no useful results

Symptoms:

- `smoke` returns zero results or irrelevant hits.
- The agent receives context but not the expected API documentation.

Checks:

```bash
nowdocs smoke <docset> "specific API or error message" --top-k 5
nowdocs doctor --docset <docset>
```

Fixes:

- Use a more specific query with API names, file names, or exact error strings.
- Re-ingest if the source documentation changed.
- If Cohere reranking is enabled, compare representative queries with the default local-only path; reranking is not guaranteed to improve every query.
- For release validation, run the real-docset evaluation gate before publishing.

## 8. Agent setup asks for registry metadata

Symptom:

- `setup plan --json` returns `registry_metadata_required`.

Fix:

```bash
nowdocs setup plan --client <client> --docset <docset> --online --json
```

The online planning step fetches registry metadata. It still does not install the docset or change client configuration. Review and approve the returned apply action separately.

## 9. Setup plan is missing, expired, stale, or tampered

Symptoms:

- `setup apply` returns `plan_not_found`, `plan_expired`, `plan_stale`, or `plan_tampered`.
- Client configuration changed after the plan was created.

Fixes:

- Do not retry the old plan hash.
- Run `setup plan` again against current state.
- Review and approve the new plan before applying it.
- Do not edit plan files or bypass target safety checks.

## 10. Client configuration requires manual action

Symptoms:

- The result code is `action_required` and the response contains redacted manual guidance.
- Codex CLI or Claude Code is absent from `PATH`, blocked by policy, ambiguous, or already has a noncanonical `nowdocs` entry.
- Cursor configuration is missing, malformed, unsafe, or already has a `nowdocs` entry.
- Claude Desktop or a generic client returns manual guidance.

Fixes:

- Follow the redacted manual guidance in the result.
- Never replace an existing `nowdocs` entry automatically.
- For Codex CLI or Claude Code, use the client's official MCP commands; do not edit its private configuration directly.
- Do not relax permissions, follow symbolic links, or create a client directory merely to make automatic setup pass.
- See [MCP Clients](MCP_CLIENTS.md) for the supported manual path.

## 11. Setup applied but verification is incomplete

Symptoms:

- The result code is `client_reload_required` or `applied_but_unverified`.
- The process exits 21 after a client change.

Fixes:

- For `client_reload_required`, reload or restart the client and run `nowdocs verify` again.
- For `applied_but_unverified`, use rollback only when the response includes a rollback object and operation ID.
- If no rollback object is present, do not invent one; inspect the client configuration manually.
- Rollback covers only the operation-owned client configuration change. It does not uninstall or downgrade the docset.
- Parse the JSON `status`, `code`, and `next_actions`. Exit code 0 alone does not prove that no action remains.

## 12. Setup rollback is rejected or was already consumed

Symptoms:

- `setup rollback` returns manual action with the observation `operation_not_recorded_by_setup`.
- A second rollback attempt is rejected after the first one succeeded.
- The user recreated a `nowdocs` registration after a successful rollback.

Expected behavior:

- Automatic rollback accepts only an operation ID returned by nowdocs for an owned client change.
- A successful rollback consumes that authorization. Replaying the old operation ID cannot remove a user-recreated registration.
- Do not retry a consumed rollback automatically and do not fabricate an operation ID.

If a valid, unconsumed rollback object was returned, use its exact operation ID once:

```bash
nowdocs setup rollback --operation-id <operation-id> --json
```

If rollback is rejected, inspect the current client registration and follow the manual guidance. Use the client's official MCP command for Codex CLI or Claude Code. For Cursor, preserve unrelated JSON entries and do not bypass path, symlink, permission, or digest checks.

## 13. Offline verification reports a missing model or docset

Symptoms:

- `verify --json` returns `model_missing`, `docset_missing`, or `docset_corrupt`.

Fixes:

```bash
nowdocs doctor --model
nowdocs install <docset>
nowdocs verify --docset <docset> --json
```

`verify` never downloads the model or repairs a docset. Preparation and repair remain explicit CLI actions.

## 14. Source build fails around Rust version or protoc

Symptoms:

- The build says a dependency requires a newer Rust compiler.
- The build fails with missing `protoc` or `google/protobuf/empty.proto`.

Fixes:

- Use the Rust version required by the current `Cargo.lock` dependency graph.
- Install `protoc` and protobuf well-known type include files.
- In constrained environments, set `PROTOC=/path/to/protoc` if the binary is not on `PATH`.

## 15. OS cache paths

nowdocs uses the OS cache directory plus `nowdocs`:

- Linux: usually `~/.cache/nowdocs` or `$XDG_CACHE_HOME/nowdocs`.
- macOS: the platform cache directory returned by the OS.
- Windows: the platform cache directory returned by the OS.

Useful commands:

```bash
nowdocs cache status
nowdocs list-installed
```

Do not manually delete active cache paths unless a maintainer specifically asks you to. Prefer `uninstall`, `doctor`, and `cache clean-staging`.
