# Integrating with Susi Core

Products build on Susi Core from the outside, through the Model Context Protocol (MCP); they
need no fork. Anyone is free to fork Susi Core under its license; the repository itself is
published one way and accepts nothing back ([CONTRIBUTING.md](../CONTRIBUTING.md)). This page
describes what the program serves today (2026-11-22); [ARCHITECTURE.md](../ARCHITECTURE.md)
says what is not finished.

## Over standard input and output

Start `susi mcp serve --root <folder>` as a child process and exchange JSON-RPC 2.0 messages
with it, one per line. The server answers `initialize`, `ping`, `tools/list` and `tools/call`;
it agrees on protocol version 2025-06-18, 2025-03-26 or 2024-11-05 and names itself
`susi-core` in `serverInfo`.

```text
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"read_file","arguments":{"path":"README.md"}}}
```

## Through the local daemon

`susi start --root <folder>` runs the same server in the background on 127.0.0.1, port 9090
plus `SUSI_PORT_OFFSET`. Send each JSON-RPC message as the body of `POST /mcp` with
`Authorization: Bearer <token>`, where the token is the content of `$SUSI_HOME/api_token`
(default `~/.susi/api_token`). `GET /health` needs no token. `susi status` says whether the
daemon runs, and `susi stop` ends it.

## The tools

All three are read-only and confined to the served folder; a path that leaves it, through
`..` or a symbolic link, is refused.

| Tool | Arguments | Returns |
|---|---|---|
| `list_directory` | `path` (optional; default the folder itself) | The entries; directories end with `/`. |
| `read_file` | `path` | The text of the file, at most 256 KiB. |
| `search_files` | `query`, `path` (optional), `max_results` (1 to 50) | Lines containing the text, ignoring case. |

## Verify your integration

`tests/mcp_runtime.rs` runs both paths end to end: it starts `susi mcp serve` and the daemon,
lists and calls the tools, and checks that a request without the token is refused. The same
exchange works against your own build.
