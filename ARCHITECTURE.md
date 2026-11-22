# Susi Core architecture

What the program does today, the packages and third-party crates it is built from, how they may
depend on each other and what is not finished. The rules behind it are in
[`AGENTS.md`](AGENTS.md); the tests that hold this page true are in
`tests/architecture_tests.rs` and `tests/lean_core_ceilings.rs`.

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

Nothing needs a setting: with no file and no variable, the folder is the current one, the home
is `~/.susi`, the port is 9090 and the token is made on first start.
[`SECURITY.md`](SECURITY.md) says what these expose and how.

## Extending the server

A host adds its own MCP tools beside the three built-in ones without editing Core: it implements
the `Tool` trait of `susi_gmcp::server` (its `tools/list` entry, and how it runs) and hands it to
`Server::with_tool`. A name already served is refused. `crates/susi-gmcp/tests/added_tools.rs`
proves it. Until 2026-11-22 that seam was a process-wide plane bus in a `susi-core` package
that no program used; it left with that package.

## Packages

Every package of the workspace, and nothing else, has a row here; a test fails when they differ.

| Package | Rust lines (`src`) | What it holds | Who uses it |
|---|---:|---|---|
| `susi` | 745 | The program: commands, daemon and installer. | The program |
| `susi-gmcp` | 582 | The MCP server, its read-only file tools and the seam for a host's own tools. | The program |
| `susi-paths` | 915 | The host directory contract: the Susi home, test isolation, standard-output guards, platform service names. | The program, Susi |
| `susi-error` | 849 | The typed error every package returns, secret redaction and the metrics sink. | The program, Susi |
| `susi-config` | 534 | Atomic file storage, JSON field helpers, progress output and random nonces. | Susi only |
| `susi-vendor-serde` | 6 | The only owner of the serde crates. | The program, Susi |
| `susi-vendor-serde-json` | 6 | The only owner of serde_json. | The program, Susi |
| `susi-vendor-sha2` | 6 | The only owner of sha2. | The program, Susi |
| `susi-vendor-getrandom` | 25 | The only owner of getrandom. | Susi only |
| `susi-vendor-libc` | 6 | The only owner of libc. | Susi only |

"Susi only": the program does not compile it; Susi, the product, builds on it (its manifest
names it), so it stays (`LIBRARIES_FOR_SUSI` in `tests/lean_core_ceilings.rs`). A package used
by neither fails that suite. Counted on 2026-11-22; `cargo core-size` prints the current total.

## Third-party crates

Every crate `Cargo.lock` resolves from outside, and nothing else, has a row here; a test fails
when they differ. Their licences are checked by `cargo deny` (`deny.toml`).

| Crate | Pulled in by | Why |
|---|---|---|
| `serde` | `susi-vendor-serde` | Typed JSON for records and messages. |
| `serde_core` | serde | Part of serde. |
| `serde_derive` | serde | Derives serde's traits at compile time. |
| `proc-macro2` | serde_derive | Compile-time only. |
| `quote` | serde_derive | Compile-time only. |
| `syn` | serde_derive | Compile-time only. |
| `unicode-ident` | proc-macro2 | Compile-time only; Unicode-3.0, the one licence exception. |
| `serde_json` | `susi-vendor-serde-json` | JSON-RPC and every JSON file. |
| `itoa` | serde_json | Part of serde_json. |
| `memchr` | serde_json | Part of serde_json. |
| `zmij` | serde_json | Part of serde_json: number formatting. |
| `sha2` | `susi-vendor-sha2` | SHA-256 for the installer's checksum. |
| `digest` | sha2 | Part of sha2. |
| `block-buffer` | sha2 | Part of sha2. |
| `crypto-common` | sha2 | Part of sha2. |
| `generic-array` | sha2 | Part of sha2. |
| `typenum` | sha2 | Part of sha2. |
| `version_check` | generic-array | Compile-time only. |
| `cpufeatures` | sha2 | Part of sha2: picks the fastest SHA-256 for the processor. |
| `cfg-if` | sha2 | Part of sha2. |
| `getrandom` | `susi-vendor-getrandom` | The system's random source, for Susi. |
| `wasi` | getrandom | Part of getrandom on WebAssembly targets; never compiled on Linux. |
| `libc` | `susi-vendor-libc`, getrandom, cpufeatures | The C library interface. |

## How packages may depend on each other

- **Leaf order.** A package depends only on packages of a lower rank: `susi-paths` and
  `susi-gmcp`, then `susi-error`, then `susi-config`, and the root program last (`LEAF_RANK` in
  `tests/architecture_tests.rs`).
- **No source mounts.** Shared code is reached through a Cargo edge, never a `#[path]`
  include across packages.
- **Third-party crates only through their facade.** Ordinary `susi-*` packages declare no
  external dependency.

## Composition roots

The root program is the only composition root: `src/main.rs` links `susi-gmcp` and `susi-paths`
and wires each command to them. Libraries never link a composition root.

## The pillars

- **GAWD** (agents): no code. Until 2026-11-22 it held Agent2Agent and Agent Client Protocol
  wire contracts that no program served; they left that day and stay in the published v0.21.72.
- **GEMI** (models): no code yet.
- **GMCP** (tools): the MCP server above, the one pillar that serves today.

## Not finished

- GAWD and GEMI have no code; a pillar returns when a program serves it.
- `toolchain/` (the standard-library builder) and `scripts/` are development code that belongs
  in Susi Tools.
