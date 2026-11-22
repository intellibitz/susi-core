# Susi Core

![SUSI Version](https://img.shields.io/badge/version-v0.21.72-blue.svg) ![License](https://img.shields.io/badge/license-Apache%202.0-green.svg)

**This repository is Susi Core, the public Apache-2.0 derivative of Susi.** It
is a local-first AI OS socket: one Rust daemon exposes models, agents and tools
through stable protocols on ports `9090-9094`.

Susi Core is one of three repositories that move together (the ecosystem page,
`ECOSYSTEM.md` in the Susi Tools repository, says how they fit):

| Name | What it is | Public? |
|------|------------|---------|
| **Susi** | The product: commercial applications that run on this socket, sold under licence. It is the primary, and the other two derive from it. Its repository is `susi` (it was `susi-commercial-apps` until 2026-11-22), and older text here calls it Susi Commercial Apps. | No. Private and licensed. |
| **Susi Core** (this repository) | The socket itself and its three core pillars: **GAWD** (agent coordination, A2A, ACP and parallel harness), **GEMI** (inference routing) and **GMCP** (MCP and governed tools). | Yes. Apache-2.0, all of it. |
| **Susi Tools** | The development tooling: whatever is used only while developing, testing and releasing Susi. None of it ships inside a production program. | Yes, since 2026-11-22. Apache-2.0; it is being assembled and holds one tool so far, a speed report. |

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
become `susi-core`; development-only code (`scripts/`, `toolchain/`, `fuzz/`)
is still in this tree and will move to Susi Tools, whose only tool so far is a
speed report; every command but `help` and `install` forwards to a runtime
host that no longer exists, so the program does not yet run on its own; and since
2026-11-22 the product holds only its licence and vault core while its apps
are rebuilt, so a capability core points to the product for has no
implementation there today.

**[Two-minute start](#two-minute-start)** · **[Host contract](#host-contract)** · **[Where it stands](#where-it-stands)** · **[Architecture](docs/architecture.md)** · **[Evidence](docs/evidence.md)** · **[Integrate your app](docs/integrating-with-susi.md)** · **[Forking and contributing](CONTRIBUTING.md)**

## What the socket is

The core is a generic protocol boundary. Concrete model engines, providers,
workflow policies and product integrations belong in separately hosted packages,
including Susi, connected through the same standard sockets.
Core ships no UI and bundles no plugin content. Existing implementations in core
are migration debt: each must have a working, compatible destination before
removal. Core has no line limit; `cargo core-size` measures the complete socket
and reports its size. Neither the architectural goal nor a passing build
certifies complete protocol conformance.

Use the socket to connect protocol clients, inspect routing decisions and run
authorized agents on the host. It is a substrate on the host, not a replacement
operating system. Susi Core is Apache-2.0 and free forever.

## Two-minute start

```bash
# 1. Start the host daemon (binds 9090-9094, seeds ~/.susi/api_token)
susi start

# 2. See what you already have: hardware, local engines, cloud keys
susi ecosystem scan
susi keys check

# 3. Run an evidence-gated mission in the current directory
susi "analyze this repository and summarize its architecture"

# 4. List models through the authenticated HTTP socket
curl -s http://127.0.0.1:9091/v1/models \
  -H "Authorization: Bearer $(cat ~/.susi/api_token)"

# 5. Ask which model susi trusts for which work, and why
susi brain                     # SUSI_BUDGET=low|balanced|max turns the cost dial
```

Every HTTP contract port needs `Authorization: Bearer <token>` except
`/health`. `susi stop` / `susi restart` are deterministic host commands.
The full command reference is in
[docs/architecture.md](docs/architecture.md#operating-the-socket-command-reference).

## Host contract

External clients can trust these ports; the daemon never silently drifts them.
Verify a running host with the external black-box conformance checker
([docs/host-contract-conformance.md](docs/host-contract-conformance.md);
`susi-conformance` lives in Susi Commercial Apps rather than this workspace).

| Port | Surface |
|------|---------|
| **9090** | GMCP / MCP HTTP (`/mcp`; `/messages` alias) |
| **9091** | GEMI HTTP: chat/completion inference (`/v1/chat/completions`, `/v1/completions`, `/v1/embeddings`, `/v1/models`), Messages (`/v1/messages`), metrics (`/metrics`), host identity (`/.well-known/susi`) |
| **9092** | A2A UDP discovery |
| **9093** | GMCP HTTP alias (streamable / SSE) |
| **9094** | A2A HTTP (`/` JSON-RPC, `/stream` SSE, `/.well-known/agent-card.json`) |

The five ports shift together, never individually: `port_offset` in
`config.json` or `SUSI_PORT_OFFSET` (which wins) moves all of them, so offset
`100` gives `9190-9194`. Swarm gossip `9095` uses the same offset but is not
part of the contract.

> **Port collisions.** When no offset is pinned and a process other than this
> instance's susi holds one of the six ports the
> daemon binds (`9090-9094` and gossip `9095`), the daemon first evicts only a
> stale susi process of its own, then moves the whole block to the first free
> offset of 100 (`9190-9195`, then `9290-9295`, up to offset `2000`). It logs
> which port was held and by which process, and publishes the chosen offset
> with its pid in `ports.json` in its config directory; `susi start`, `susi os
> status` and every client of the same instance find it there, and the start
> banner says the block is a fallback. The ports never move one at a time and
> never become ephemeral. To choose the block yourself, pin it: a
> `SUSI_PORT_OFFSET` value (`0` included) or a non-zero `port_offset` in
> `config.json` is bound exactly, and the daemon refuses to start if that block
> is taken. A second instance on one host also needs its own root:
> `SUSI_HOME=~/.susi-b SUSI_PORT_OFFSET=100 susi start`.

## Where it stands

The host exposes the five-port contract with bearer authentication and a
conformance checker. Current paths include evidence-gated missions, tool
receipts, an HMAC audit chain and a Wasm interpreter. These paths have separate
capability and qualification checks; availability does not establish complete
standards compliance or qualify an artifact for production.

The current default build uses an external inference plug or configured HTTP
engine/provider. Embeddings and browser page capture are plug ports and report
a typed capability gap when no plug is wired. In-process model delivery remains
a documented gap. Pattern-based governance vetoes are not a sandbox boundary.
See [current limitations](docs/evidence.md) and [delivery acceptance](docs/step1-acceptance.md).

Full acceptance requires the exact combined source and artifact to pass the
installed journey, promotion, health and rollback checks. The separate warm
`xtask fast-release` budget is <10 s; it does not replace full release
qualification. The cold compile target is <60 s. Report current-tree measurements
and platform coverage explicitly; retained historical timings are not current
qualification. Core releases target native x86_64 Linux only (since 2026-11-18). Windows
desktop apps, installers and runtime distribution live in Susi Commercial Apps;
other platforms remain portable source without core release qualification.

## Install

Agents obtain an identified native Linux `x86_64` artifact and invoke its Rust
installer with `susi install`. It discovers the user profile, verifies the staged
executable, and installs without replacing a different existing executable. No
shell script, hosting-service connection or user-written configuration is
required for placement.
Susi never downloads a model by itself: cloud models work right away, and
`susi os manage model install starter` downloads the licensed local starter
model (about 429 MB) when you ask. A local goal without it says so and names
that command.
Native installation does not by itself qualify a candidate for production;
qualification, promotion and rollback have separate evidence gates.

From source on `x86_64` Linux, install with `install.sh` from a clone of this
repository (Windows installers and desktop binaries live in Susi Commercial
Apps; `install.ps1` remains in this tree only as legacy migration debt). This
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
checksums are described in [docs/releases.md](docs/releases.md). Platform scope
and qualification are recorded in [docs/evidence.md](docs/evidence.md#platforms).

### Developing susi

Agents own authorized development, verification, integration and production
qualification under the [active engineering contract](AGENTS.md). Work in an
isolated worktree with disjoint scopes (`local-workflow start <agent>
--scope <path>`, `xtask local-build debug`; core's development and release tools
are still built from Susi Commercial Apps' `tools/core-xtask`, and move to Susi
Tools later) and private build/runtime state. One page says where a feature
belongs and how an agent lands one here without being asked:
[docs/landing-features.md](docs/landing-features.md).

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
artifact-bound release provenance. The [workflow guide](docs/evidence.md#building-susi-the-workflow)
describes available commands and migration limits.

## Read next

- [docs/README.md](docs/README.md): every document, indexed.
- [docs/landing-features.md](docs/landing-features.md): why agents land features unprompted, where a feature belongs (Susi, Susi Core or Susi Tools) and the loop that lands one here.
- [docs/architecture.md](docs/architecture.md): pillars, Tier S invariants, plane bus and its loopback rendezvous, implementation services and protocol adapters, the lean-core direction, the command reference.
- [docs/evidence.md](docs/evidence.md): verified claims, platforms, the delivery workflow and every admitted gap.
- [ARCHITECTURE.md](ARCHITECTURE.md): the enforceable crate-boundary contract (`tests/architecture_tests.rs`).
- The MCP specification: upstream, in the `modelcontextprotocol/modelcontextprotocol` repository. Core kept a pinned copy of it, and Susi a GMCP certification built on that copy, until 2026-11-22, when both were removed; history keeps them.
- [docs/lean-core.md](docs/lean-core.md#complete-socket-size): how core's size is counted by category (first-party runtime Rust apart from tests, tooling, vendored sources and audit trails), measured and printed by [`tests/lean_core_ceilings.rs`](tests/lean_core_ceilings.rs). Core has no line limit: `cargo core-size` reports the number and never fails on it.
- [docs/extension-pack.md](docs/extension-pack.md), [docs/harness.md](docs/harness.md), [VISION.md](VISION.md), [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Apache License 2.0. See [LICENSE](LICENSE), [NOTICE](NOTICE), and [REUSE.toml](REUSE.toml). Provided as-is, without warranty; users run the engine and its agents at their own risk.

The repository is published one way: the designated publisher alone holds push
authority, and authorized agents act within that scope. This grants no additional
write access; no outside pull request, patch or commit is accepted. You are free
to fork under the license, which grants code rights and no right to the Susi name
or marks. See [CONTRIBUTING.md](CONTRIBUTING.md).
