# Lean core: measured and reported

Susi Core is a small, generic socket: one program, `susi`, that serves the Model Context
Protocol over standard input and output and through a local daemon ([`ARCHITECTURE.md`](../ARCHITECTURE.md)).
Its three pillars are GAWD (agents), GEMI (models) and GMCP (tools); a pillar holds code when a
program serves it, and today only GMCP does. Concrete implementations belong in separately
hosted packages, including Susi, reached through standard protocols.

The [active engineering contract](../AGENTS.md) governs current work. This page says how core's
size and reach are measured and what the speed budget is; it is not a progress log.

## Nothing is there for nobody

Two tests keep the workspace lean without a limit on its size:

- **Every package is used.** `tests/lean_core_ceilings.rs` walks what the program `susi`
  compiles (normal and build edges, from `cargo metadata`). A package the program does not
  compile fails the suite unless Susi, the product, builds on it and it is named in
  `LIBRARIES_FOR_SUSI` and in `ARCHITECTURE.md`'s package table.
- **Every dependency is named.** `tests/architecture_tests.rs` requires `ARCHITECTURE.md` to name
  every package of the workspace and every crate `Cargo.lock` resolves, with what pulls it in
  and why, and nothing else. A new dependency fails until its reason is written down.

On 2026-11-22 the first of these found four packages and five facades that no program compiled
and no Susi manifest named (5,400 lines); they left, and `Cargo.lock` fell from 89 packages to
33.

## Complete socket size

Core has no line limit. Size is measured and reported, because a number nobody reads cannot
inform a decision; no size refuses a change.

```sh
cargo core-size
```

This runs one test in [`tests/lean_core_ceilings.rs`](../tests/lean_core_ceilings.rs) that
counts the complete socket and prints the total and each package's share. It fails only when
the measurement itself is broken, for example when a package that must exist is counted as
empty. Read the current measurement from the command rather than adding dated measurements to
this document.

Counting contract: physical Rust lines, including comments, blanks, inline tests, every platform
and every feature in first-party runtime files, plus the `susi-vendor-*` facades. Test files and
development tooling are reported apart. Count tracked and non-ignored untracked files. The
source report is not the size of linked dependencies or executable bytes. Keep the report
honest: do not relocate tests, minify, or exclude compiled code to change the number it prints.

## Recorded baselines

The same test file records five counts that are not line counts:

- third-party crates the program compiles;
- workspace packages the program compiles;
- workspace members declared in `Cargo.toml`;
- packages resolved in `Cargo.lock`;
- vendor facades the program's package names directly.

Each is a baseline, not a ceiling. A change may raise one in the same commit: raise the
constant in `tests/lean_core_ceilings.rs` to the new count and say why, in the constant's
comment and in the commit body. A change that lowers a count lowers its baseline too, so the
record stays true. A count above its baseline fails the ordinary run, and the failure says
exactly this.

## Preserve behavior

Preserve what the program serves: its commands, the supported MCP revisions and transports,
its typed errors and its compatibility fields. Before removing an implementation that a
program serves, demonstrate its working destination over the standard socket. Code that no
program serves or reads is not a capability; the compiler deciding that nothing uses it is the
proof, and the removal says so in its commit. UI and bundled product or plug-in content do not
belong in core. Keep proprietary source out of core.

## Speed budget

Nothing waits more than 10 seconds warm (V-42): the build, the whole test suite, the lint and
each release gate, measured and dated, cold timings apart. A step over the line is a defect to
fix, never met by skipping, loosening or narrowing a test. Susi's `docs/speed.md` holds the
measurements.

The fast loop does not replace release qualification. A release is qualified against the exact
source, lock file, toolchain and target, with every gate and a smoke run of the built program
(`docs/releases.md`).

## Records and publication

Operational history, completed tasks, build measurements and evidence belong outside
checked-out source. Do not append wave logs, dated mandates or progress tables here.

Core is published one way. The designated publisher alone holds push authority; authorized
agents exercise it within scope without gaining new access. Outside pull requests, patches and
commits are not accepted. Everyone is free to fork under Apache-2.0; code from a fork returns
only through authorized source admission with its pinned provenance and required notices. See
[CONTRIBUTING.md](../CONTRIBUTING.md) and the harvest rule in [AGENTS.md](../AGENTS.md).
