# MCP Client Configuration

nowdocs exposes two read-only MCP tools over stdio: `nowdocs_search` and `nowdocs_list`. Register `nowdocs serve`; do not add host or port flags.

## Capability matrix

The current machine-readable matrix is available from `nowdocs capabilities --json`:

| Client | Detect | Generate | Apply | Verify | Managed behavior |
|---|---|---|---|---|---|
| Claude Code | supported | supported | conditional | conditional | Uses the official `claude mcp` CLI at user scope. |
| Claude Desktop | supported | conditional | unsupported | unsupported | Provides MCPB guidance only; no legacy JSON writes. |
| Codex CLI | supported | supported | conditional | conditional | Uses the official `codex mcp` CLI for the global registration. |
| Cursor | supported | supported | conditional | conditional | Uses guarded, rollback-aware updates to an existing approved-root MCP JSON file. |
| Generic MCP client | unsupported | supported | unsupported | unsupported | Generates deterministic manual configuration. |

`conditional` means runtime preconditions must pass. nowdocs fails closed when it cannot prove that an automatic change is safe.

## Prepare nowdocs

Before configuring a client, prepare one docset and the pinned model:

```bash
nowdocs doctor --model
nowdocs install nextjs
nowdocs verify --docset nextjs --json
```

`verify` is offline and never downloads the model, which is why model preparation is explicit.

For agent-managed setup, use the plan/approval/apply flow instead:

```bash
nowdocs setup plan --client <CLIENT_ID> --docset nextjs --online --json
nowdocs setup apply --plan-hash <APPROVED_PLAN_HASH> --json
nowdocs verify --docset nextjs --client <CLIENT_ID> --json
```

See [Agent Setup](AGENT_SETUP.md) before automating these commands.

The apply action may use the network to fetch its pinned docset package and is not fully reversible as a combined operation. A returned rollback object covers only an operation-owned client configuration change; it does not uninstall or downgrade the docset.

A successful setup rollback consumes its operation authorization. Replaying the same operation id cannot remove or restore a matching registration that the user creates later.

## Claude Code

Client id: `claude-code`

Managed setup uses the official Claude Code CLI and never parses or writes `~/.claude.json` directly. It checks for an existing entry first, refuses to replace one, and adds a new server only at user scope.

The equivalent manual command is:

```bash
claude mcp add --transport stdio --scope user nowdocs -- /absolute/path/to/nowdocs serve
```

Inspect it with:

```bash
claude mcp get nowdocs
nowdocs verify --docset nextjs --client claude-code --json
```

If `claude` is not on `PATH`, policy blocks it, or the existing entry is ambiguous, nowdocs returns manual guidance without changing the configuration.

## Codex CLI

Client id: `codex`

Managed setup uses the official Codex CLI. nowdocs never reads, parses, or writes `~/.codex/config.toml` or another Codex-owned file. It probes the global registration with:

```bash
codex mcp get nowdocs --json
```

When that command proves that the entry is absent, the generated canonical add command is:

```bash
codex mcp add nowdocs -- /absolute/path/to/nowdocs serve
```

nowdocs refuses to replace an existing, malformed, disabled, or otherwise ambiguous entry. Verification requires the exact absolute binary and the single `serve` argument. Setup-owned rollback rechecks that exact canonical entry before it invokes:

```bash
codex mcp remove nowdocs
```

If the Codex CLI is missing or any probe is ambiguous, nowdocs returns manual guidance without changing Codex configuration.

## Cursor

Client id: `cursor`

Managed setup targets the global `.cursor/mcp.json` under the approved user root. It can add a canonical `nowdocs` entry only when:

- the root, parent directory, and target are real safe paths rather than symbolic links;
- the target file already exists and contains valid JSON;
- `mcpServers` is an object or absent;
- no `nowdocs` entry already exists;
- the target has not changed since planning.

The update preserves unrelated top-level fields and MCP servers, creates an operation-owned backup, and uses crash-safe replacement. A successful file verification still returns `client_reload_required` because nowdocs cannot claim that the Cursor UI has reloaded it. Automatic rollback is digest-guarded and refuses to restore over a later user edit.

If the file is missing, malformed, unsafe, or already contains `nowdocs`, configure it manually with the generic JSON below. nowdocs does not create the `.cursor` directory automatically.

## Claude Desktop

Client id: `claude-desktop`

The current supported local-server path for Claude Desktop uses a Desktop Extension (`.mcpb`) added by the user through **Settings > Extensions**. nowdocs does not yet ship a signed, cross-platform MCPB.

The adapter therefore provides guidance only. It never writes `claude_desktop_config.json`, never claims that an extension is installed or loaded, and does not offer automatic verification. Treat a nowdocs MCPB as a separate future deliverable.

## Generic MCP client

Client id: `generic`

Use this configuration with a client that accepts the common MCP JSON shape:

```json
{
  "mcpServers": {
    "nowdocs": {
      "command": "/absolute/path/to/nowdocs",
      "args": ["serve"]
    }
  }
}
```

Use an absolute binary path when the client starts with a restricted `PATH`. Generic setup is generation-only: nowdocs does not locate, edit, or verify an unknown client's configuration.

Native Cohere reranking is optional and must be configured in the environment of the process that starts `nowdocs serve`, not in an MCP tool request. Desktop clients can launch with a restricted environment, so make the required variables available to that client or server process. Read the [native Cohere reranking guide](RERANKING.md) before enabling it.

## Verification checklist

Before debugging a client, verify the local server and docset:

```bash
nowdocs status --json
nowdocs doctor --mcp
nowdocs verify --docset nextjs --json
```

After configuring a supported client, run:

```bash
nowdocs verify --docset nextjs --client <CLIENT_ID> --json
```

Expected MCP tools:

- `nowdocs_search`
- `nowdocs_list`

`nowdocs_search` requires an explicit `docset`. If a client omits it, correct the prompt or tool call rather than weakening the server schema.
