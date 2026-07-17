# Agent Setup

This guide defines the safe machine-to-machine setup flow introduced in nowdocs v0.2.0. It is designed for coding agents that can run local commands but must leave state-changing decisions to the user.

## Safety model

- MCP remains read-only. It exposes only `nowdocs_search` and `nowdocs_list` over stdio.
- `capabilities`, `status`, and `verify` are read-only and offline-safe.
- `setup plan` may fetch registry metadata when `--online` is present. It stores a private, expiring plan, but it does not install a docset or change client configuration.
- `setup apply` changes local state and always requires explicit user approval.
- A plan hash protects the integrity and scope of a stored plan; it is not cryptographic evidence that the user consented to apply it.
- A rollback command is valid only when the apply result returns a rollback object. It restores only an operation-owned client configuration change; it does not uninstall or downgrade a docset. Never invent an operation id.
- JSON output is the contract. Parse `status` and `code`; exit code 0 can still mean that a manual or reload action is required.

## Discover capabilities

Run this before assuming that a command or client action is supported:

```bash
nowdocs capabilities --json
```

The response reports the agent-contract schema version, MCP protocol and tools, command properties, client capabilities, and security boundaries. Treat `conditional` client capabilities as runtime-dependent, not as a promise that nowdocs will modify every installation.

## Inspect local state

```bash
nowdocs status --json
```

This command observes the cache layout, installed docsets, pinned model state, and automation metadata without creating files, loading the model, or using the network.

## Create a setup plan

Start offline:

```bash
nowdocs setup plan --client codex --docset nextjs --json
```

If the response code is `registry_metadata_required`, rerun the exact next action with network access:

```bash
nowdocs setup plan --client codex --docset nextjs --online --json
```

`--online` fetches and validates registry metadata so the plan can pin the selected package. It does not install the package or edit client configuration.

## Review and apply a plan

Before applying, show the user at least:

- the client and docset;
- the reported risk and whether network access or a client write is planned;
- the exact `next_actions[].argv` returned by nowdocs;
- the declared target paths or target identifiers, if any;
- whether the combined action is fully reversible;
- the plan hash.

For `setup-apply`, treat the returned disclosure conservatively. A plan that installs or updates a docset reports network access because apply may fetch the selected package. The combined action is not fully reversible: rollback may restore an operation-owned client configuration change, but it does not undo docset work.

Recommended approval prompt:

> nowdocs prepared a setup plan for `<client>` and `<docset>`. It reports risk `<risk>` and network access `<network_access>`. Applying it may install or update the docset and may update the client configuration. The combined action is not fully reversible. Approve `nowdocs setup apply --plan-hash <hash>`?

Do not apply merely because planning succeeded. Do not modify the returned argv, substitute another plan hash, or bypass a manual/conflict result.

### Apply the approved plan

```bash
nowdocs setup apply --plan-hash <plan-hash> --json
```

Interpret the result by `code`:

| Code | Meaning | Agent action |
|---|---|---|
| `setup_complete` | Docset and supported client checks completed. | Continue to verification. |
| `client_reload_required` | Configuration was verified on disk, but the client may not have reloaded it. | Ask the user to reload or restart the client, then verify again. |
| `action_required` | nowdocs safely declined automatic work. | Present the redacted observations and manual guidance; do not force a write. |
| `applied_but_unverified` | A write committed, but complete verification or setup metadata persistence failed. | Use the returned rollback only if one is present; otherwise ask the user to verify manually. |
| `plan_not_found`, `plan_expired`, `plan_stale`, or `plan_tampered` | The plan cannot be safely reused. | Create a new plan. Do not retry the old hash. |

When the response contains `rollback`, retain that object only for the current operation. It covers only the owned client configuration change. A later user edit can make automatic rollback unsafe, in which case nowdocs refuses it.

## Verify the result

```bash
nowdocs verify --docset nextjs --client codex --json
```

`verify` performs read-only local retrieval and optionally verifies the client adapter. It never downloads the embedding model. If it returns `model_missing`, explicitly prepare the model with:

```bash
nowdocs doctor --model
```

Then run `verify` again. A `client_reload_required` result is not proof that the client UI has reloaded the server.

## Roll back a setup-owned change

If an apply response includes a rollback object, use its exact operation id:

```bash
nowdocs setup rollback --operation-id <operation-id> --json
```

Rollback is guarded by operation metadata and content digests. It does not uninstall or downgrade the docset. If the client configuration changed after setup, nowdocs refuses to overwrite the later edit and returns manual guidance. A successful rollback consumes its setup-owned authorization before a later identical user configuration can be affected. Replaying the same operation id returns `operation_not_recorded_by_setup` and does not invoke the client adapter again.

## Client capability matrix

| Client id | Detect | Generate | Apply | Verify | Setup behavior |
|---|---|---|---|---|---|
| `claude-code` | supported | supported | conditional | conditional | Uses the official `claude mcp` CLI at user scope and refuses to replace an existing `nowdocs` entry. |
| `claude-desktop` | supported | conditional | unsupported | unsupported | Returns truthful MCPB guidance. It never edits the legacy Desktop JSON file. |
| `codex` | supported | supported | conditional | conditional | Uses the official `codex mcp` CLI and never reads or edits Codex configuration files directly. |
| `cursor` | supported | supported | conditional | conditional | Can safely update an existing approved-root `.cursor/mcp.json`; unsafe, missing, malformed, or conflicting targets require manual action. |
| `generic` | unsupported | supported | unsupported | unsupported | Generates deterministic stdio configuration for manual installation. |

`conditional` means the adapter acts only after it proves its runtime preconditions. Codex CLI and Claude Code use their official CLIs rather than editing client-owned configuration files. Cursor uses only the explicitly approved root and refuses unsafe or ambiguous files. Claude Desktop and generic clients remain guidance-only. See [MCP Clients](MCP_CLIENTS.md) for exact commands and recovery boundaries.

## Machine-readable results and exit classes

The JSON `code` is authoritative within an exit class. Exit 0 can still carry `action_required` or `client_reload_required`, so an agent must never use process success alone as permission to continue.

| Exit | Class | Examples |
|---:|---|---|
| 0 | Completed or safely action-required | `ready`, `already_satisfied`, `setup_complete`, `action_required`, `client_reload_required` |
| 2 | Invalid request | `invalid_request` |
| 10 | Plan or concurrency conflict | `plan_not_found`, `plan_expired`, `plan_stale`, `plan_tampered`, `operation_in_progress` |
| 20 | Goal not completed | Missing/corrupt state, client conflict, network failure, or verification failure |
| 21 | Applied but not fully verified | `applied_but_unverified` |
| 30 | Policy refusal | `config_write_unsafe`, `unsupported_platform` |

1. Read exactly one JSON document from stdout for each `--json` command.
2. Keep stderr separate; update reminders and diagnostics are not part of the JSON envelope.
3. Validate `schema_version` before depending on fields.
4. Branch on `status` and `code`, not on summary text.
5. Execute only an argv returned by a recognized `next_actions` entry and only after satisfying its `requires_confirmation` flag.
6. Never include local paths, configuration bytes, environment values, or credentials in chat transcripts.
7. Never call state-changing CLI commands through MCP; writable MCP tools do not exist.

## Recommended agent policy

Replace the two placeholders before giving this prompt to a coding agent:

```text
Set up nowdocs for client <CLIENT_ID> with docset <DOCSET> on this machine.

Safety rules:
1. Run `nowdocs capabilities --json` and `nowdocs status --json` first. Do not assume a capability that is not reported.
2. Create the setup plan offline first. If and only if nowdocs returns `registry_metadata_required`, rerun the returned planning action with `--online`.
3. Do not edit any MCP client configuration directly. Do not run install, apply, rollback, or another state-changing command outside the nowdocs plan contract.
4. Show me the client, docset, exact `next_actions[].argv`, risk, network access, reversibility, and plan hash. Then stop and wait for my explicit approval of the exact apply command.
5. After approval, run only the approved `nowdocs setup apply --plan-hash ... --json` command. Parse `status` and `code`; do not treat exit code 0 as unconditional success.
6. Run `nowdocs verify --docset <DOCSET> --client <CLIENT_ID> --json`. This verification must remain offline. If the model is missing, report the explicit `nowdocs doctor --model` action instead of running it without approval.
7. If a result requires a client reload, ask me to reload it. If a rollback object is returned, explain that it covers only the operation-owned client configuration change, report the exact rollback command, and do not run it without separate approval. Never invent an operation id.
8. Keep stdout JSON separate from stderr. Do not reveal credentials, environment values, configuration bytes, or local absolute paths in your report.
```

### Manual fallback

If the installed binary does not expose `capabilities` or `setup`, follow [Getting Started](GETTING_STARTED.md) and [MCP Clients](MCP_CLIENTS.md). Manual setup remains supported for legacy v0.1.2 binaries and clients that do not expose a managed adapter.
