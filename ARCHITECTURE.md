# Susi Core architecture

What the program does today, the crates it is built from, how they may depend on each other
and what is not finished. The rules behind it are in [`AGENTS.md`](AGENTS.md); the tests that
hold the layering are in `tests/architecture_tests.rs`.

## What runs

One program, `susi` (`src/main.rs`):

- `susi mcp serve [--root <dir>]` serves the Model Context Protocol over standard input and
  output (`crates/susi-gmcp`). It answers `initialize` (protocol versions 2025-06-18,
  2025-03-26 and 2024-11-05), `ping`, `tools/list` and `tools/call`, with three read-only tools
  confined to one folder: `list_directory`, `read_file` (at most 256 KiB) and `search_files`
  (at most 50 results; it skips `.git`, `target`, `node_modules` and `.susi`).
- `susi start [--root <dir>]`, `susi status` and `susi stop` run the same server as a local
  daemon (`src/daemon.rs`) on 127.0.0.1, port 9090 plus `SUSI_PORT_OFFSET`: `GET /health` is
  open; `POST /mcp` and `POST /shutdown` need the bearer token in `$SUSI_HOME/api_token`.
- `susi install` copies the program into `$SUSI_HOME/bin` (`src/cli/install_cli.rs`) and never
  replaces a different program already there.

[`SECURITY.md`](SECURITY.md) says what these expose and how.

## The crates

| Crate | Rust lines (`src`) | What it holds |
|---|---:|---|
| `susi-paths` | 915 | The host directory contract on Linux: the Susi home, test isolation, standard output guards. |
| `susi-error` | 849 | `EaiError`, the typed error every crate returns. |
| `susi-config` | 534 | Configuration wire types, atomic file storage and host inspection. |
| `susi-abi` | 2,108 | Wire and record types shared by the planes: frames, cells, routing, memory, evidence. |
| `susi-gawd-acp` | 429 | Agent Client Protocol wire contracts (GAWD pillar). |
| `susi-gawd-a2a` | 28 | Agent2Agent protocol constants; re-exports the ACP contracts (GAWD pillar). |
| `susi-core` | 2,722 | `plane_bus`, the in-process control plane between feature planes, and `plane_bus_ipc`, its framing across processes; agent types, a bounded command runner and scoped tools. |
| `susi-gmcp` | 549 | The MCP server and its read-only file tools (GMCP pillar). |
| `susi-vendor-*` | 79 | Ten facades, each the only owner of one third-party crate. |
| `susi` (root) | 746 | The program: commands, daemon and installer. |

Counted on 2026-11-22 at the commit that wrote this page; `cargo core-size` prints the
current total.

## How crates may depend on each other

- **Leaf order.** A crate depends only on crates of a lower rank: `susi-paths` and `susi-abi`,
  then `susi-error`, `susi-config`, `susi-gawd-acp`, `susi-core`, `susi-gmcp`, `susi-gawd-a2a`,
  and the root program last (`LEAF_RANK` in `tests/architecture_tests.rs`).
- **The feature planes do not depend on each other.** The pillars talk only through
  `plane_bus` in `susi-core`, which is the one control plane between them; the layer matrix in
  `tests/architecture_tests.rs` lists the edges each crate may not have.
- **No source mounts.** Shared code is reached through a Cargo edge, never a `#[path]`
  include across crates.
- **Third-party crates only through their facade.** Ordinary `susi-*` crates declare no
  external dependency.

## Composition roots

The root program is the only composition root: `src/main.rs` links `susi-gmcp` and
`susi-core` and wires each command to them. Libraries never link a composition root.

## The pillars

- **GAWD** (agents): the A2A and ACP wire contracts, with no server in core yet.
- **GEMI** (models): no code yet.
- **GMCP** (tools): the MCP server above, the one pillar that serves today.

## Not finished

- No handler is registered on `plane_bus` in the program yet: the daemon calls the MCP server
  directly.
- A2A and ACP have contracts but no server; GEMI has no code.
- `toolchain/` (the standard-library builder) and `scripts/` are development code that belongs
  in Susi Tools.
