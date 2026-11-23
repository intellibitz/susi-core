# Susi Core architecture

What this repository holds, the packages and third-party crates it is built from, how they may
depend on each other and what is not finished. The rules behind it are in
[`AGENTS.md`](AGENTS.md); the tests that hold this page true are in
`tests/architecture_tests.rs` and `tests/lean_core_ceilings.rs`.

## What it is

Susi Core is the small set of libraries that Susi, the product, builds on. It has no program:
until 2026-11-22 it had one, `susi`, which served the Model Context Protocol with three read-only
file tools and ran a local daemon. No feature of Susi used it, and that day it left with its
server package (`susi-gmcp`), its installer, its release records and their tests. The last
release with the program, v0.21.72, stays published and tagged.

Every package here is one Susi's manifest names, and each row below says what Susi uses it for.
A package nothing uses fails `tests/lean_core_ceilings.rs`.

## Packages

Every package of the workspace, and nothing else, has a row here; a test fails when they differ.

| Package | Rust lines (`src`) | What it holds | Who uses it |
|---|---:|---|---|
| `susi` | 0 | The root package: no code; it holds this repository's own tests. | These tests |
| `susi-paths` | 101 | The platform-service table (the owner-only file opener `susi-error`'s sink uses off Unix) and a command launcher that opens no console window. | Susi on Windows: its Windows crate fills the platform-service table at start-up, and one of its tests launches through the command launcher |
| `susi-error` | 247 | An append-only, owner-only JSONL metrics sink. | Susi on Windows: one test of its Windows crate writes to the metrics sink |
| `susi-vendor-serde` | 6 | The only owner of the serde crates. | Susi's vault: the sealed store's records |
| `susi-vendor-serde-json` | 6 | The only owner of serde_json. | Susi's vault: the sealed store's JSON |
| `susi-vendor-sha2` | 6 | The only owner of sha2. | Susi's vault: key fingerprints and the master key's form |
| `susi-vendor-getrandom` | 25 | The only owner of getrandom. | Susi's vault: keys and nonces |

These are `LIBRARIES_FOR_SUSI` in `tests/lean_core_ceilings.rs`. Counted on 2026-11-23;
`cargo core-size` prints the current total.

## Third-party crates

Every crate `Cargo.lock` resolves from outside, and nothing else, has a row here; a test fails
when they differ. Their licences are checked by `cargo deny` (`deny.toml`).

| Crate | Pulled in by | Why |
|---|---|---|
| `serde` | `susi-vendor-serde` | Typed JSON records. |
| `serde_core` | serde | Part of serde. |
| `serde_derive` | serde | Derives serde's traits at compile time. |
| `proc-macro2` | serde_derive | Compile-time only. |
| `quote` | serde_derive | Compile-time only. |
| `syn` | serde_derive | Compile-time only. |
| `unicode-ident` | proc-macro2 | Compile-time only; Unicode-3.0, the one licence exception. |
| `serde_json` | `susi-vendor-serde-json` | Every JSON file. |
| `itoa` | serde_json | Part of serde_json. |
| `memchr` | serde_json | Part of serde_json. |
| `zmij` | serde_json | Part of serde_json: number formatting. |
| `sha2` | `susi-vendor-sha2` | SHA-256. |
| `digest` | sha2 | Part of sha2. |
| `block-buffer` | sha2 | Part of sha2. |
| `crypto-common` | sha2 | Part of sha2. |
| `generic-array` | sha2 | Part of sha2. |
| `typenum` | sha2 | Part of sha2. |
| `version_check` | generic-array | Compile-time only. |
| `cpufeatures` | sha2 | Part of sha2: picks the fastest SHA-256 for the processor. |
| `cfg-if` | sha2 | Part of sha2. |
| `getrandom` | `susi-vendor-getrandom` | The system's random source. |
| `wasi` | getrandom | Part of getrandom on WebAssembly targets; never compiled on Linux. |
| `libc` | getrandom, cpufeatures | The C library interface. |

## How packages may depend on each other

- **Leaf order.** A package depends only on packages of a lower rank: `susi-paths`, then
  `susi-error`, and the root package last (`LEAF_RANK` in `tests/architecture_tests.rs`).
- **No source mounts.** Shared code is reached through a Cargo edge, never a `#[path]`
  include across packages.
- **Third-party crates only through their facade.** Ordinary `susi-*` packages declare no
  external dependency.

## Not finished

- **What Susi uses is thin.** Susi's vault uses the four facades; `susi-paths` and `susi-error`
  are reached only by Susi's Windows crate, and `susi-error` only by one of its tests. Whether
  they stay is decided with a check on a Windows host; Susi's own plan
  (`docs/foundation-plan.md` there, "The fourth round") records it.
- The Git hooks are one-line calls to `susi-tools hook <stage>`; their rules are Susi Tools' (since
  2026-11-23).
