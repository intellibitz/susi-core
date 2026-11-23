# Lean core: measured and reported

Susi Core is the small set of libraries Susi, the product, builds on
([`ARCHITECTURE.md`](../ARCHITECTURE.md)). It has no program since 2026-11-22.

The [active engineering contract](../AGENTS.md) governs current work. This page says how core's
size and reach are measured and what the speed budget is; it is not a progress log.

## Nothing is there for nobody

Two tests keep the workspace lean without a limit on its size:

- **Every package is used.** `tests/lean_core_ceilings.rs` fails on a workspace package that
  is not one Susi builds on (named in `LIBRARIES_FOR_SUSI` and in `ARCHITECTURE.md`'s package
  table, with what Susi uses it for), apart from the root package that holds the tests.
- **Every dependency is named.** `tests/architecture_tests.rs` requires `ARCHITECTURE.md` to name
  every package of the workspace and every crate `Cargo.lock` resolves, with what pulls it in
  and why, and nothing else. A new dependency fails until its reason is written down.

## Complete size

Core has no line limit. Size is measured and reported, because a number nobody reads cannot
inform a decision; no size refuses a change.

```sh
cargo core-size
```

This runs one test in [`tests/lean_core_ceilings.rs`](../tests/lean_core_ceilings.rs) that
counts the libraries and prints the total and each package's share. It fails only when the
measurement itself is broken, for example when a package that must exist is counted as empty.
Read the current measurement from the command rather than adding dated measurements to this
document.

Counting contract: physical Rust lines, including comments, blanks, inline tests, every platform
and every feature in first-party library files, plus the `susi-vendor-*` facades. Test files
and development tooling are reported apart. Count tracked and non-ignored untracked files. Keep
the report honest: do not relocate tests, minify, or exclude compiled code to change the number
it prints.

## Recorded baselines

The same test file records two counts that are not line counts:

- workspace members declared in `Cargo.toml`;
- packages resolved in `Cargo.lock`.

Until 2026-11-23 it also recorded what the root package compiles; that package has no code, so
those counts measured nothing.

Each is a baseline, not a ceiling. A change may raise one in the same commit: raise the
constant in `tests/lean_core_ceilings.rs` to the new count and say why, in the constant's
comment and in the commit body. A change that lowers a count lowers its baseline too, so the
record stays true. A count above its baseline fails the ordinary run, and the failure says
exactly this.

## Preserve behavior

Preserve what Susi uses: the libraries' public items it calls, their typed errors and their
behaviour. Susi stops using a library before it leaves; code nothing uses is not a capability,
and the compiler deciding that nothing uses it is the proof, which the removal states in its
commit. UI and bundled product or plug-in content do not belong in core. Keep proprietary source
out of core.

## Speed budget

Nothing waits more than 10 seconds warm (V-42): the build, the whole test suite and the lint,
measured and dated, cold timings apart. A step over the line is a defect to fix, never met by
skipping, loosening or narrowing a test. Susi's `docs/speed.md` holds the measurements.

## Records and publication

Operational history, completed tasks, build measurements and evidence belong outside
checked-out source. Do not append wave logs, dated mandates or progress tables here.

Core is published one way. The designated publisher alone holds push authority; authorized
agents exercise it within scope without gaining new access. Outside pull requests, patches and
commits are not accepted. Everyone is free to fork under Apache-2.0; code from a fork returns
only through authorized source admission with its pinned provenance and required notices. See
[CONTRIBUTING.md](../CONTRIBUTING.md) and the harvest rule in [AGENTS.md](../AGENTS.md).
