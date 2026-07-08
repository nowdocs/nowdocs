# MCP Client Configuration

nowdocs exposes MCP over stdio. Use `nowdocs serve`; do not add host or port flags.

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

## Cursor

Add a server entry equivalent to:

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

Restart Cursor after changing MCP configuration. Then ask the agent to list tools and verify `nowdocs_search` and `nowdocs_list` are available.

## Claude Code

Configure nowdocs as a stdio MCP server with command `nowdocs` and args `serve`:

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

Run `nowdocs doctor --mcp` locally if Claude Code cannot see the tools.

## Claude Desktop

Use the same stdio server shape in the Claude Desktop MCP configuration file:

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

Restart Claude Desktop after editing config.

## Aider

Use Aider's MCP configuration mechanism to register a stdio server:

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

Confirm the docset exists before asking Aider to search:

```bash
nowdocs list-installed
nowdocs smoke <docset> "installation configuration example"
```

## Verification checklist

Before debugging a client, verify the server locally:

```bash
nowdocs doctor
nowdocs doctor --mcp
nowdocs list-installed
nowdocs smoke <docset> "installation configuration example"
```

Expected tools:

- `nowdocs_search`
- `nowdocs_list`

If a client calls `nowdocs_search` without `docset`, fix the client prompt or tool call. `docset` is required.
