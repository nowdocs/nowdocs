# MCP Client Configuration

nowdocs exposes MCP over stdio. Register `nowdocs serve`; do not add host or port flags.

## Before configuring a client

Confirm that the client can resolve `nowdocs` from its environment and that a docset is installed:

```bash
nowdocs doctor --model
nowdocs install nextjs
nowdocs smoke nextjs "middleware matcher configuration"
```

If a desktop client starts with a restricted `PATH`, use an absolute path to the `nowdocs` binary in its MCP configuration.

## Generic MCP JSON

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

Native Cohere reranking is optional and must be configured in the environment
of the process that starts `nowdocs serve`, not in an MCP tool request. Desktop
clients can launch with a restricted environment, so make the required
variables available to that client or server process. Read the [native Cohere
reranking configuration and data-transfer disclosure](../README.md#optional-native-cohere-reranking)
before enabling it.

## Cursor

Add the generic server entry to Cursor's MCP configuration, restart Cursor, and ask the agent to list tools. Verify that `nowdocs_search` and `nowdocs_list` are available.

## Claude Code

Register a stdio MCP server with command `nowdocs` and arguments `serve`. If Claude Code cannot see the tools, run this local check first:

```bash
nowdocs doctor --mcp
```

## Claude Desktop

Use the generic JSON shape in the Claude Desktop MCP configuration file, then fully restart Claude Desktop after saving the file.

## Aider

Use Aider's MCP configuration mechanism to register the same stdio server. Before searching, confirm the docset is present:

```bash
nowdocs list-installed
nowdocs smoke nextjs "middleware matcher configuration"
```

## Verification checklist

Before debugging any client, verify the server locally:

```bash
nowdocs doctor
nowdocs doctor --mcp
nowdocs list-installed
nowdocs smoke nextjs "middleware matcher configuration"
```

Expected MCP tools:

- `nowdocs_search`
- `nowdocs_list`

`nowdocs_search` requires an explicit `docset`. If a client calls it without one, update the client prompt or tool call.
