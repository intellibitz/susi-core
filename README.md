# Susi Core

![License](https://img.shields.io/badge/license-Apache%202.0-green.svg)

**This repository is Susi Core, the public Apache-2.0 derivative of Susi.** It
is the small set of Rust libraries that Susi, the product, builds on: home and
path resolution, an owner-only metrics sink, and the wrappers through which Susi uses serde,
serde_json, sha2 and getrandom. It has no program.

Susi Core is one of three repositories that move together (the ecosystem page,
`ECOSYSTEM.md` in the Susi Tools repository, says how they fit):

| Name | What it is | Public? |
|------|------------|---------|
| **Susi** | The product: secure autonomy for a company's AI agents, sold under licence. It is the primary, and the other two derive from it. Its repository is `susi` (it was `susi-commercial-apps` until 2026-11-22). | No. Private and licensed. |
| **Susi Core** (this repository) | The libraries Susi builds on, and nothing else: each is named in [ARCHITECTURE.md](ARCHITECTURE.md) with what Susi uses it for. | Yes. Apache-2.0, all of it. |
| **Susi Tools** | The development tooling: whatever is used only while developing, testing and releasing Susi. None of it ships inside a production program. | Yes, since 2026-11-22. Apache-2.0; its README lists what it holds. |

Nothing in this repository is secret, and it builds and tests without the
private product. Dependencies flow one way, from Susi to Susi Core and Susi
Tools, never the reverse.

The software is built by AI agents. The repository is published one way: no
outside pull request, patch or commit is merged. Anyone is free to fork it
under Apache-2.0; the licence grants rights to the code, not to the Susi name
or marks.

Not finished yet, said plainly: what Susi uses here is thin (its vault uses the
four wrappers; `susi-paths` and `susi-error` are reached only by Susi's Windows
crate). The development-only code moved to Susi Tools: the Git hooks here are
one-line calls to `susi-tools hook <stage>`.

**[What it holds](#what-it-holds)** · **[Architecture](ARCHITECTURE.md)** · **[Security](SECURITY.md)** · **[Forking](CONTRIBUTING.md)**

## What it holds

Core holds only what Susi's features use. A library enters when a feature of
Susi needs it and it is useful without a licence; a library no feature of Susi
uses leaves. Core ships no UI and bundles no plugin content. Core has no line
limit; `cargo core-size` measures the libraries and reports their size.

Until 2026-11-22 Core also had a program, `susi`: a Model Context Protocol
server with read-only file tools and a local daemon. No feature of Susi used
it, so it left that day with its server package, installer and release
records. Its last release, `v0.21.72`, stays published and tagged, and history
keeps its source.

## Using the libraries

Susi builds on a pinned commit of this repository's `main` through Cargo's Git
dependencies; nothing is published to a package registry and there are no
releases. The libraries build and test on x86_64 Linux with the toolchain in
`rust-toolchain.toml`, offline, from a clone:

```sh
cargo nextest run --workspace --offline --locked
```

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
a task closes only when its acceptance passes on the actual combined tree, and
published tags remain immutable.

Operational history, completed tasks and growing evidence belong outside source,
in a local database when available; absent history never proves acceptance.
Keep only necessary active coordination while the native tooling needs it.
Update the current contract in place instead of extending mandate histories.
Preserve mandatory licenses and attribution. [AGENTS.md](AGENTS.md) names the
commands.

## Read next

- [docs/README.md](docs/README.md): every document, indexed.
- [AGENTS.md](AGENTS.md): the contract, how agents land features here unprompted and where a feature belongs (Susi, Susi Core or Susi Tools).
- [ARCHITECTURE.md](ARCHITECTURE.md): the libraries, what Susi uses each for and how they may depend on each other (held by `tests/architecture_tests.rs`).
- [SECURITY.md](SECURITY.md): what the libraries touch and how to report a vulnerability.
- [docs/lean-core.md](docs/lean-core.md#complete-size): how core's size is counted by category, measured and printed by [`tests/lean_core_ceilings.rs`](tests/lean_core_ceilings.rs). Core has no line limit: `cargo core-size` reports the number and never fails on it.
- [VISION.md](VISION.md), [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Apache License 2.0. See [LICENSE](LICENSE), [NOTICE](NOTICE), and [REUSE.toml](REUSE.toml). Provided as-is, without warranty.

The repository is published one way: the designated publisher alone holds push
authority, and authorized agents act within that scope. This grants no additional
write access; no outside pull request, patch or commit is accepted. You are free
to fork under the license, which grants code rights and no right to the Susi name
or marks. See [CONTRIBUTING.md](CONTRIBUTING.md).
