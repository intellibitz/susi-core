# Security Policy

## Supported Versions

Security updates are applied to the latest release on `main` (`0.21.x`).

| Version | Supported |
| ------- | --------- |
| `0.21.x` (latest `main`) | :white_check_mark: |
| `< 0.21.0` | :x: |

## What the program exposes

Read from `src/daemon.rs` and `crates/susi-gmcp` on 2026-11-22. The program connects to no
other host (its `start`, `status` and `stop` commands reach the daemon on 127.0.0.1) and
starts no other program (`susi start` starts only itself).

- **`susi mcp serve`** talks only over standard input and output, with the process that
  started it. It trusts that process.
- **The daemon** (`susi start`) listens on 127.0.0.1 only, port 9090 plus `SUSI_PORT_OFFSET`.
  `GET /health` answers without a token and returns the service name, version and process ID.
  `POST /mcp` and `POST /shutdown` need `Authorization: Bearer <token>`, compared in constant
  time. The token is 32 random bytes from `/dev/urandom`, written as hex to
  `$SUSI_HOME/api_token` (default `~/.susi`) on first start, readable by its owner only
  (mode 0600 when the file is created). A request's headers may be at most 16 KiB and its
  body at most 1 MiB; a connection that sends nothing for 10 seconds is dropped.
- **The MCP tools are read-only** and confined to one folder (`--root`, default the current
  one): every path is resolved, symbolic links included, and refused unless it stays inside
  that folder. `read_file` returns at most 256 KiB; `search_files` returns at most 50 results,
  skips files over 1 MiB and does not enter `.git`, `target`, `node_modules` or `.susi`.
- **Files the program writes:** `$SUSI_HOME/api_token` and `$SUSI_HOME/daemon.json` (the
  daemon's process ID, port, folder and version), and `susi install` writes the program into
  `$SUSI_HOME/bin`, never replacing a different program already there.

**Limits, said plainly.** Anyone who can read your files can read the token. The daemon
answers one connection at a time, so a local client that holds a connection open delays others
by up to 10 seconds. An `api_token` file that already exists keeps its permissions when it is
rewritten.

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report suspected vulnerabilities privately via [GitHub Private Vulnerability Reporting](https://github.com/intellibitz/susi-core/security/advisories/new) on `intellibitz/susi-core`.

Please include:
- A clear description of the vulnerability and the affected part (e.g., the daemon in
  `src/daemon.rs`, the MCP server in `susi-gmcp`, `susi-paths`, `susi-config`).
- Minimal reproduction steps or a proof-of-concept test case.
- Potential impact (e.g., bearer-token bypass, path traversal outside the served folder, or a
  file read beyond the limits above).

We aim to acknowledge vulnerability reports within **48 hours** and publish a verified fix and regression test as quickly as possible.
