# Lean core: a generic standards socket, measured and reported

Susi core is a generic, standards-protocol AI OS socket. Its three pillars are
GAWD (A2A and ACP), GEMI and GMCP. The socket host and foundation are part of
core. Concrete implementations belong in separately hosted packages, including
Susi, reached through standard
protocols. Preserve capabilities while moving their implementations.

The [active engineering contract](../AGENTS.md) governs current work. This
page defines how core's size is measured and the performance budgets; it is
not an append-only progress log or evidence of a completed migration.

## Complete socket size

Core has no line limit. The earlier limit of 9,999 physical runtime Rust
lines, and the rule that a change adding lines removes as many, were withdrawn
on 2026-11-22. Size is still measured and reported, because a number nobody
reads cannot inform a decision; no size refuses a change.

Print the measurement:

```sh
cargo core-size
```

This runs one test in
[`tests/lean_core_ceilings.rs`](../tests/lean_core_ceilings.rs) that counts the
complete socket and prints the total and each package's share. It does not
fail on the number. It fails only when the measurement itself is broken, for
example when a package that must exist is counted as empty. A run that
executes no test has measured nothing. Read the current measurement from the
command rather than adding dated measurements to this document.

Counting contract: physical Rust lines, including comments, blanks, inline
tests, every platform and every feature in first-party runtime files, plus
the `susi-vendor-*` facades. Count all pillar, foundation, host, client and
compatibility code, rather than only the crate named `susi-core`. Reuse the
existing path-based category rules. The unconditionally public
`crates/susi-paths/src/test_env.rs` is runtime and must remain counted. Separately
report test/bench/fuzz files, development tooling, third-party `vendor/`
source and `.agents/` records. Count tracked and non-ignored untracked files;
deduplicate Git's file list. The source report is not the size of linked
dependencies or executable bytes. Keep the report honest: do not relocate
tests, minify, or exclude compiled code to change the number it prints.

## Recorded baselines

The same test file records five counts that are not line counts:

- third-party crates the three measurement roots compile;
- workspace crates the three measurement roots compile;
- workspace members declared in `Cargo.toml`;
- packages resolved in `Cargo.lock`;
- vendor facades the pillar crates name directly.

A sixth, the reviewed unsafe-baseline files under `.agents/baseline/`, left on
2026-11-22 with those files: nothing read them once the unsafe-code check left
with the private repository's tools.

Each is a baseline, not a ceiling. A change may raise one in the same commit:
raise the constant in `tests/lean_core_ceilings.rs` to the new count and say
why, in the constant's comment and in the commit body. A change that lowers a
count lowers its baseline too, so the record stays true. A count above its
baseline fails the ordinary run, and the failure says exactly this. What is
measured did not change when the rule did.

```sh
cargo test --locked --test lean_core_ceilings -- --nocapture
```

Earlier separate per-pillar, category and per-file line ceilings remain
removed. Dependency reach uses normal and build edges from the three
measurement roots `susi-gawd-a2a`, `susi-gawd-acp` and `susi-gmcp`, the GAWD and
GMCP crates (GEMI has none yet); development edges are excluded.

## Preserve behavior through extraction

Core retains generic protocol contracts, authorization boundaries, discovery,
transport, cancellation and socket composition. Move concrete provider,
workflow, tool, model and application implementations to independently hosted
packages. UI and bundled product or plugin content do not belong in core.
Do not relabel implementations or move them into an excluded counting category.

Preserve public APIs, CLI actions, the five-port contract, supported MCP
revisions and transports, ACP and A2A behavior, inference compatibility,
Wasm admission, permissions, cancellation, persistence and harness execution.
Protocol identifiers and compatibility fields stay intact. Runtime and current
documentation remain identity-neutral; required license and attribution
notices stay with the source they cover.

Before removing an implementation, demonstrate its working destination and
the installed core adapter over the standard socket. Exercise original and
replacement behavior, including typed errors, authorization refusals, uncertain
mutations, process ownership, cancellation, large payloads and native path
handling. Inspect every platform and feature caller. A finite test corpus
must not conceal a known capability gap. Keep proprietary source out of core.

A size report and structural socket tests do not establish standards
compliance. Run affected behavioral regressions, relevant live conformance
checks and `cargo check --workspace --all-targets` on the combined migration
tree. Keep unmet behavior and conformance requirements open.

## Compile budget

The cold compile target is under 60 seconds for `cargo build --offline --locked
-p susi` in a fresh target directory on the current native qualification host.
Measure wall time, total and workspace CPU-seconds, critical path, host load,
toolchain, target, feature set and source revision. Report cold and warm runs
separately; cache reuse does not establish a cold result.

No tool measures it today: `xtask build-time`, which did, left with the private
repository's `tools/core-xtask` on 2026-11-22. Historical measurements may be
absent. Keep results and comparisons in local
storage rather than extending this source document with successive runs.

## Warm build, test and release budget

The warm pipeline budget is under 10 seconds: `total_ms < 10_000`, where
`total_ms = build_ms + test_ms + release_ms`. Measure the core build, meaningful
verification tests sharing compatible artifacts, and emission of the candidate
artifact with its SHA-256 provenance manifest. Avoid redundant profile builds
while keeping debug, candidate and production artifacts distinct.

The fast warm pipeline does not replace full release qualification. Qualify
releases against the exact source, lockfile, toolchain, target and feature set,
with required tests, license and native-code audits, executable checks and
artifact digest. Preserve required provenance with the artifact, independently
of optional operational history. Retain atomic promotion, health checks and
rollback; source publication alone does not qualify a release.

## Records and publication

Operational history, completed tasks, build measurements, facts and evidence
belong outside checked-out source, in a local database when available; history
may be lost. Do not append wave logs, dated mandates or progress tables here.
Keep current directives and minimal active coordination readable, removing
active records after accepted completion. Missing records never prove accepted
work, a completed dependency or a qualified release. Live authorization,
recovery and spend-enforcement state retain their correctness requirements.

Work in an owned isolated worktree, respect active scopes, and fetch and merge
reachable `origin/main` before committing. Publish meaningful checkpoints about
every ten minutes through normal fast-forward updates. `main` remains an
unqualified development stream. Agent handoffs use `refs/agents/<agent>`;
release candidates and immutable snapshots retain their separate qualification.

Core is published one way. The designated publisher alone holds push authority;
authorized agents exercise it within scope without gaining new access. Outside
pull requests, patches and commits are not accepted. Everyone is free to fork
under Apache-2.0; code from a fork returns only through authorized source
admission with its pinned provenance and required notices. See
[CONTRIBUTING.md](../CONTRIBUTING.md) and the harvest rule in
[AGENTS.md](../AGENTS.md).
