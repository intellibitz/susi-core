# Susi Core

![SUSI Version](https://img.shields.io/badge/version-v0.21.72-blue.svg) ![License](https://img.shields.io/badge/license-Apache%202.0-green.svg)

**This repository is Susi Core, the public Apache-2.0 derivative of Susi.** It
is a local-first AI OS socket: a small Rust program that serves the Model
Context Protocol on your own machine, over standard input and output or from a
local daemon on 127.0.0.1.

Susi Core is one of three repositories that move together (the ecosystem page,
`ECOSYSTEM.md` in the Susi Tools repository, says how they fit):

| Name | What it is | Public? |
|------|------------|---------|
| **Susi** | The product: commercial applications that run on this socket, sold under licence. It is the primary, and the other two derive from it. Its repository is `susi` (it was `susi-commercial-apps` until 2026-11-22), and older text here calls it Susi Commercial Apps. | No. Private and licensed. |
| **Susi Core** (this repository) | The socket itself and its three core pillars: **GAWD** (agents: no code yet), **GEMI** (models: no code yet) and **GMCP** (tools: the MCP server that runs today, which a host extends with its own tools). | Yes. Apache-2.0, all of it. |
| **Susi Tools** | The development tooling: whatever is used only while developing, testing and releasing Susi. None of it ships inside a production program. | Yes, since 2026-11-22. Apache-2.0; it is being filled, and its README lists what it holds. |

Nothing in this repository is secret, and it builds and runs without the
private product. A capability that lives in the product is not here: the core
says so and names what serves it. Dependencies flow one way, from Susi to Susi
Core and Susi Tools, never the reverse.

The software is built by AI agents. The repository is published one way: no
outside pull request, patch or commit is merged. Anyone is free to fork it
under Apache-2.0; the licence grants rights to the code, not to the Susi name
or marks.

Not finished yet, said plainly: this repository is `susi-core` since
2026-11-22 (it was `susi`), but its program is still named `susi` and will
become `susi-core`; development-only code (`scripts/`, `toolchain/`)
is still in this tree and will move to Susi Tools, which is being filled; the
program's own runtime is small (an MCP server and a local daemon) and grows one
working capability at a time; and since
2026-11-22 the product holds only its licence and vault core while its apps
are rebuilt, so a capability core points to the product for has no
implementation there today.

**[What runs today](#what-runs-today)** · **[Where it stands](#where-it-stands)** · **[Architecture](ARCHITECTURE.md)** · **[Security](SECURITY.md)** · **[Integrate your app](docs/integrating-with-susi.md)** · **[Forking and contributing](CONTRIBUTING.md)**

## What the socket is

The core is a generic protocol boundary. Concrete model engines, providers,
workflow policies and product integrations belong in separately hosted packages,
including Susi, connected through the same standard sockets.
Core ships no UI and bundles no plugin content. Existing implementations in core
are migration debt: each must have a working, compatible destination before
removal. Core has no line limit; `cargo core-size` measures the complete socket
and reports its size. Neither the architectural goal nor a passing build
certifies complete protocol conformance.

Today it gives an MCP client read-only access to one folder on the host; the
rest of the socket grows one working capability at a time. It is a substrate on
the host, not a replacement operating system. Susi Core is Apache-2.0 and free forever.

## What runs today

Since 2026-11-22 the program runs on its own (Linux x86_64):

```bash
# An MCP server over standard input and output, with read-only file tools
# confined to one folder: list_directory, read_file, search_files.
susi mcp serve --root ~/my-project

# The same tools from a local daemon on 127.0.0.1:9090 (plus SUSI_PORT_OFFSET):
susi start --root ~/my-project     # starts it in the background
curl -s http://127.0.0.1:9090/health
curl -s http://127.0.0.1:9090/mcp -H "Authorization: Bearer $(cat ~/.susi/api_token)" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'
susi status
susi stop
```

`susi mcp serve` speaks MCP 2025-06-18 (and 2025-03-26, 2024-11-05), so an MCP
client such as Claude Code can use it directly (`claude mcp add susi -- susi
mcp serve --root .`). A path that leaves the root, by `..`, an absolute name or
a symbolic link, is refused. The daemon listens on 127.0.0.1 only; `/health`
needs no token, `/mcp` and `/shutdown` need the bearer token in
`~/.susi/api_token` (made on first start, readable by its owner only).

That is the whole command list (`susi --help`). Until 2026-11-22 the program
also listed 55 commands that only handed their work to a runtime host of the
private product; they left with that host. The runtime grows from here one
working capability at a time.
## Where it stands

The program serves what [What runs today](#what-runs-today) lists and nothing
more. Core releases target native x86_64 Linux only (since 2026-11-18); other
platforms remain portable source without core release qualification.

The newest release, `v0.21.72` (2026-11-22), is the program above: it was cut
with Susi Tools' `susi-tools release` command, rebuilt that day after the earlier
release tools left the private product ([docs/releases.md](docs/releases.md)).
The warm
build, test and release budget is under 10 seconds and the cold compile target
under 60 seconds; Core's Linux gates take 16 to 36 seconds today.

## Install

Agents obtain an identified native Linux `x86_64` artifact and invoke its Rust
installer with `susi install`. It discovers the user profile, verifies the staged
executable, and installs without replacing a different existing executable. No
shell script, hosting-service connection or user-written configuration is
required for placement.
Susi never downloads anything by itself.
Native installation does not by itself qualify a candidate for production;
qualification, promotion and rollback have separate evidence gates.

From source on `x86_64` Linux, install with `install.sh` from a clone of this
repository (`install.ps1` remains in this tree only as legacy migration debt). This
needs Git and a Rust toolchain from [rustup](https://rustup.rs): the release tag
pins its compiler in `rust-toolchain.toml` (for example, 1.99.0), and when that
toolchain is missing the installer stops and prints the `rustup toolchain
install` command. It never installs software for you and downloads no binary.

Replace `REPOSITORY_URL` below with the selected canonical Git repository URL
(local bare repo, self-hosted Git remote, or read-only mirror).

```sh
git clone REPOSITORY_URL susi && cd susi && ./install.sh
```

The installer builds the newest release tag of the clone's `origin` (or of
`SUSI_SOURCE_REPO`, the core repository's git URL, when it runs outside a
clone; `SUSI_RELEASE_TAG` pins a release) with the settings in that release's
committed record, and installs it only when it reports the tag's version. A
cold build takes several minutes. Releases, their records and the reference
checksums are described in [docs/releases.md](docs/releases.md). It installs
`v0.21.72` today; a cold build of it took 48 seconds on the development host.

### Developing susi

Agents own authorized development, verification, integration and production
qualification under the [active engineering contract](AGENTS.md), one page that
says how work lands here (an isolated worktree, a claimed task, the Linux gates,
a fast-forward of `main`, all through the `susi-tools` development program from
the Susi Tools repository) and where a feature belongs.

Use portable Git transport with zero dependency on GitHub or GitLab (canonical
local or self-hosted Git remote; the copies on GitHub and GitLab are backups
the project pushes to and never fetches from) and reconcile the current remote
before work and commits. Commit, push,
merge and pull meaningful work about every ten minutes.
Each commit carries `Task: T-<AGENT>-<n>`. Main is unqualified development;
release integration verifies the actual combined tree before a release ref
moves, and published release tags remain immutable. Keep source publication
separate from production promotion, health checks and rollback.

Operational history, completed tasks and growing evidence belong outside source,
in a local database when available; absent history never proves acceptance.
Keep only necessary active coordination while the native tooling needs it.
Update the current contract in place instead of extending mandate histories.
Preserve mandatory licenses, attribution, live authorization/recovery data and
artifact-bound release provenance. [AGENTS.md](AGENTS.md) names the commands.

## Read next

- [docs/README.md](docs/README.md): every document, indexed.
- [AGENTS.md](AGENTS.md): the contract, how agents land features here unprompted and where a feature belongs (Susi, Susi Core or Susi Tools).
- [ARCHITECTURE.md](ARCHITECTURE.md): what runs, the crates and how they may depend on each other (held by `tests/architecture_tests.rs`).
- [SECURITY.md](SECURITY.md): what the program exposes, its limits and how to report a vulnerability.
- The MCP specification: upstream, in the `modelcontextprotocol/modelcontextprotocol` repository. Core kept a pinned copy of it, and Susi a GMCP certification built on that copy, until 2026-11-22, when both were removed; history keeps them.
- [docs/lean-core.md](docs/lean-core.md#complete-socket-size): how core's size is counted by category (first-party runtime Rust apart from tests, tooling, vendored sources and audit trails), measured and printed by [`tests/lean_core_ceilings.rs`](tests/lean_core_ceilings.rs). Core has no line limit: `cargo core-size` reports the number and never fails on it.
- [VISION.md](VISION.md), [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Apache License 2.0. See [LICENSE](LICENSE), [NOTICE](NOTICE), and [REUSE.toml](REUSE.toml). Provided as-is, without warranty; users run the engine and its agents at their own risk.

The repository is published one way: the designated publisher alone holds push
authority, and authorized agents act within that scope. This grants no additional
write access; no outside pull request, patch or commit is accepted. You are free
to fork under the license, which grants code rights and no right to the Susi name
or marks. See [CONTRIBUTING.md](CONTRIBUTING.md).
