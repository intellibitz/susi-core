# Forking and contributing to Susi

Susi core is open source under the [Apache License, Version 2.0](LICENSE). It is
**published one way** (publication policy, Mandate 72 in
[`AGENTS.md`](AGENTS.md)).

## 1. How this repository is published

- **One publisher.** The repository's designated publisher alone holds push
  authority. Authorized agents may exercise that authority within their scope;
  this policy grants no additional write access.
- **Nothing is accepted from outside.** No pull request, patch or commit from
  an outside party is reviewed, accepted or merged, and no outside party is
  given write access. Please do not open one: it will not be merged.
- **You are free to fork.** Fork the repository and work in your fork. You need
  no permission and no agreement beyond the license.

This is a statement about how the repository is run. It takes nothing away from
the rights the license gives you.

## 2. What the license lets you do

The [Apache License 2.0](LICENSE) lets anyone copy, use, change and
redistribute this code, in source or binary form, for any purpose including a
commercial one. When you redistribute it, the license asks you to:

1. give recipients a copy of the license;
2. mark the files you changed as changed;
3. keep the copyright, patent, trademark and attribution notices of the source
   you took, and the content of any `NOTICE` file it carries.

The license grants code rights, not rights to names. "Susi" and "Susi
Certified" are IntelliBitz marks (Apache-2.0 section 6): a fork must not present
itself as the official Susi or as certified by IntelliBitz. The code comes as
is, without warranty.

## 3. Use the libraries without forking

Most uses need no fork. Core is the small set of libraries Susi builds on
([`AGENTS.md`](AGENTS.md), "The libraries and their size"); any Rust project
may depend on a commit of this repository through Cargo's Git dependencies,
under Apache-2.0. Core has no program, and there are no releases or registry
packages.

## 4. Working in your fork

A fork may change anything. If you want your fork to stay close to this
repository, these are the invariants its own tests enforce
(`tests/architecture_tests.rs`, `tests/core_docs_promise_no_ui_or_plugins.rs`):

1. **Libraries only.** Core ships no UI (no HTML, CSS, JS, GUI or TUI crates)
   and bundles no third-party plugin catalogs.
2. **Zero `unsafe` and zero production panics.** Workspace crates forbid or
   deny `unsafe_code`. Production code returns a typed `Result`; `unwrap()`,
   `expect()`, `panic!()`, `todo!()` and `unimplemented!()` are denied by
   Clippy unless an inline `#[allow(...)]` carries a written justification.
3. **Facade dependency rule.** `susi-*` crates declare zero direct third-party
   dependencies; each external crate is wrapped once in a `susi-vendor-*`
   facade crate.
4. **Permissive licenses only.** Every distributed dependency is `Apache-2.0`,
   `MIT`, `BSD-2-Clause` or `BSD-3-Clause` (`cargo deny check licenses`).

### Setup

```bash
git clone https://github.com/<you>/susi.git
cd susi
git config core.hooksPath .githooks
```

The hooks in `.githooks/` are one line each: they run `susi-tools hook <stage>`
from Susi Tools (build and install `susi-tools` from that repository), which
checks staged Rust formatting, secret-shaped lines and large blobs at commit,
and this repository's own push rules at push. A fork may replace them.

### Checks

```bash
# 1. Formatting
cargo fmt --all --check

# 2. Lints
cargo clippy --workspace --all-targets --locked -- -D warnings

# 3. Architecture and documentation contract tests
cargo test --test core_docs_promise_no_ui_or_plugins --locked
cargo test --test architecture_tests --locked

# 4. License compliance
cargo deny --locked check licenses
```

### A live, isolated host

Never test against your own `~/.susi` state. Start an isolated instance with
`SUSI_HOME` and `SUSI_PORT_OFFSET`:

```bash
SUSI_HOME=/tmp/susi-dev SUSI_PORT_OFFSET=700 cargo run -- start
cargo run -p susi-conformance -- --port-offset 700 --token-file /tmp/susi-dev/api_token
SUSI_HOME=/tmp/susi-dev SUSI_PORT_OFFSET=700 cargo run -- stop
```

## 5. If your fork has something Susi wants

Nothing flows back by request. Susi takes existing Apache-2.0 work by its own
harvest: a pinned commit, per-file license verification, preserved notices and
recorded attribution (the harvest rule in [AGENTS.md](AGENTS.md)). A fork of Susi is an
Apache-2.0 project like any other, so that is the only way its code can return,
and it is Susi's decision.

## 6. Reporting a security problem

Follow [SECURITY.md](SECURITY.md). A report is information, not a contribution:
the fix is written and published by authorized agents.

## 7. Authorized agents

Authorized agents follow [AGENTS.md](AGENTS.md): agent roots, the
Susi Board and task claims. They use `susi-tools workflow check`, `tasks
start`, `ship` and `tasks finish` from an isolated worktree, reconcile with
`origin/main` at session start, on resume and before committing, and label
commits with a `Task:` trailer.

`main` is the only branch; Susi builds on a pinned commit of it, and Core cuts
no releases. See [`AGENTS.md`](AGENTS.md), "Publication and Git" and
"Releases".

## 8. License

This repository is licensed under the
[Apache License, Version 2.0](LICENSE).
