# Releases

A Susi core release is two things in git and nothing else: an annotated tag
`vX.Y.Z`, and the committed release record `releases/<tag>.json` on the
default branch. No hosting service's release page, asset list or API is part of
a release (decision of 2026-11-09: hosted copies are storage only). Canonical
development and release transport use zero-GitHub portable Git storage (a local
or self-hosted bare Git remote, with an optional one-way public mirror of
`refs/heads/main` and signed `refs/tags/v*` only), so the repositories never
depend on GitHub.

Core standalone releases target native `x86_64` Linux only (since 2026-11-18);
Windows is not a Core target. Binaries are not committed and are not downloaded
from GitHub. Every core install and upgrade builds the tag from source on the
host that will run it (decision of 2026-11-09).

## The record: `releases/<tag>.json`

Schema `susi-release-record/v1`. `tests/release_records.rs` checks every
record on every gate run.

| Field | Meaning |
|---|---|
| `schema` | `susi-release-record/v1` |
| `product`, `version`, `tag` | `susi`, `X.Y.Z`, `vX.Y.Z`; the file is named after the tag |
| `commit` | the full commit id the tag points at; installers refuse a tag that points elsewhere |
| `tagged` | the annotated tag's date |
| `build.toolchain` | the Rust toolchain of the tag's `rust-toolchain.toml` |
| `build.host` | the host triple the reference binaries were built on |
| `build.profile`, `build.features`, `build.env` | the cargo profile, the feature list and the `NAME=value` environment the reference build used; installers build with exactly these |
| `build.command` | the full command, for people |
| `reproducible` | `false` today: no byte-for-byte reproducibility is claimed |
| `assets[]` | one line per reference binary: `file`, `target`, `package`, `bin`, `sha256` |
| `missing_targets[]` | any released target with no reference build and why (active core releases target `x86_64` Linux only; historical records through `v0.21.71` also listed `x86_64` Windows before Windows distribution moved to Susi Commercial Apps) |
| `checks` | the checks run on the reference build, what passed, what did not run and why; `locally_qualified` says whether the release was qualified on its build host |
| `provenance` | where each fact in the record came from |

The installer reads the record with line tools, because a fresh host may have
no JSON parser. The test therefore also pins the layout: the scalar fields sit on
lines of their own, `env` is one line, each asset is one line, and every value
passed to cargo is a plain word.

### Which release a host installs

A record lists every released target either with a reference build under
`assets` or under `missing_targets`, so a release whose record has no reference
build for a host was not qualified there. `install.sh` builds the newest tag,
or the one `SUSI_RELEASE_TAG` names.

**Upgrading is running `install.sh` again**: it builds the newest release from
its tag and replaces the installed program only when the build reports the
tag's version. There is no automatic updater today. The earlier one,
`scripts/susi-release-sync.sh` with its hourly timer, still drove the program of
before 2026-11-22 (a `restart` command and a health port the program no longer
has), so it could not update today's program; it left on 2026-11-22 and returns
only when it works with the program it updates.

### What the sha256 means

The `sha256` of an asset is the digest of the **reference build**: the binary
built when the release was cut, with the recorded command, settings and
toolchain, on the recorded host. It identifies that binary. It is not a gate
for anyone else's build: your own build of the same tag may differ in bytes
(paths, linker, toolchain build, host). Installers report whether a build
matches the reference digest, and they never refuse a build for differing.
**The gate is the version check**: the built binary must report the tag's
version, and the tag must point at the record's commit.

## Cutting a release

A release is cut with Susi Tools' development program, on a native x86_64 Linux
host (from Windows, inside WSL2), in a clean checkout of `main` whose
`Cargo.toml` already carries the new version:

1. `susi-tools release check` runs every check that comes before the gates and
   changes nothing: the checkout is Susi Core, clean and at the remote's `main`;
   the tag is `v` plus the workspace version; the version is neither tagged here
   nor published on the remote, and it has no records yet.
2. `susi-tools release qualify --task <id> [--sign]` runs the gates on that
   exact commit, each logged with its digest: formatting, `cargo check`, Clippy
   with warnings denied, licences (`cargo deny`), the whole suite
   (`cargo nextest`), doc-tests, and the reference build with the profile,
   features and environment the installer will repeat. Then it runs the built
   program from a private home, recorded as the `smoke` gate: `--version`, an
   MCP session over standard input and output, and the daemon's health, token
   check and stop. A failing gate writes nothing. When everything passes it
   commits `releases/qualification/<tag>.json` alone on top of the qualified
   commit, creates the annotated tag on that commit (`--sign` makes it a signed
   tag; the storage sync publishes only signed `v*` tags to the public copies),
   and commits `releases/<tag>.json`, which names the tagged commit; both
   commits name the task. Nothing leaves the machine.
3. `susi-tools release publish vX.Y.Z` pushes the release record's commit to
   `main` and the tag in one atomic push, so the tag and its records are
   published together or not at all, and refuses a version that is already
   published. Core's pre-push hook hands release tags to `push-policy` where it
   is installed, which admits a tag only on a commit that adds nothing but its
   qualification record, with every required gate at exit 0 and the tests gate
   carrying its counts.

From a Windows host the gates run in WSL2, where no signing key is: qualify
there without `--sign`, bring the two commits and the tag over, and sign the tag
on the host that holds the key, keeping its message, tagger and date (so the
record's `tagged` time stays true), before publishing.

The first release cut this way is `v0.21.72` (2026-11-22): every gate and the
smoke run passed on `f5a0cde7b` in 20 seconds (380 tests across 67 binaries),
the signed tag names `2f8522f5c`, which adds only its qualification record, and
a cold `install.sh` from the canonical server rebuilt it in 48 seconds with a
checksum matching the reference build. Until 2026-11-22 these steps were the
private repository's `release-qualify`, which left with its `tools/core-xtask`;
the `push-policy` program built from it is still installed on the development
host and still checks every release tag.

After a release is published, `install.sh` installs it (see below).

## Distilled releases

A record says where its tree came from. `source: snapshot`, or no `source` at
all (v0.21.54 and v0.21.55, written before the field existed), is a release cut
from a snapshot of main. `source: distill` is a release cut from a distilled
candidate with `release-qualify --from-distill <name>`: the candidate's receipt
had the verdict `accept_verified_candidate`, every commit made on the Git host
was listed in `adopted_host_commits`, and every included task's acceptance had
exited 0 on the composed tree. Such a record carries a `distillation` block:
`candidate`, `receipt_path`, `receipt_sha256`, `envelope_commit`,
`base_commit`, `final_commit`, `final_tree`, `included_tasks` and
`adopted_host_commits`, so it says which tasks the release contains.

## Installing and upgrading

Core installation and upgrades target native `x86_64` Linux; on Windows, run
the installer inside WSL2 (the Windows installer, `install.ps1`, left on
2026-11-22). The installer resolves the newest tag with `git ls-remote --tags` against a
configured zero-GitHub canonical or mirror Git remote, check that tag out, read
its record from the remote's default branch, build `susi` natively with the
recorded profile, features and environment, and install the result only when it
reports the tag's version.

They need git and a Rust toolchain installed through rustup (plus a local or
self-hosted Cargo crate mirror when building completely offline). The tag's
`rust-toolchain.toml` pins the compiler (for example, 1.99.0); when that
toolchain is not installed the installer stops and prints the
`rustup toolchain install` command. It never installs system software.

| Path | Remote |
|---|---|
| `install.sh` (Linux x86_64; install and upgrade) | `SUSI_SOURCE_REPO`, else the `origin` of the checkout the script runs from |

`SUSI_RELEASE_TAG` pins a specific release. There is no built-in default
remote: the installer that finds none says which variable to set.

## Commercial certification

Susi's `scripts/refresh-certified-core.sh` fetches the core
repository over git (its configured core remote) and takes the pinned
release's commit and reference digests from `releases/<tag>.json`.
