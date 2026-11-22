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

The scripts read the record with line tools, because a fresh host may have no
JSON parser. The test therefore also pins the layout: the scalar fields sit on
lines of their own, `env` is one line, each asset is one line, and every value
passed to cargo is a plain word.

### Which release a host installs

The installed Susi is the stable copy and tracks the latest release qualified
for the host it runs on. A record lists every released target
either with a reference build under `assets` or under `missing_targets`, so a
release whose record has no reference build for a host was not qualified there.
`scripts/susi-release-sync.sh` therefore installs the newest tag whose record
has a reference build for this host and skips newer ones that do not; `--check`
prints both. An explicit `--tag` of a release not qualified for the host is
refused unless `SUSI_RELEASE_ALLOW_UNQUALIFIED_HOST=1` is set knowingly.

**Keeping it updated is a duty, not only a report** (cutting or publishing a
release qualified for the current host must sync `~/.susi/bin/susi` (`susi.exe`)
and the live daemon before the release is considered complete). Required: the
installed copy is behind the latest release qualified for the host. Feasible:
that release is qualified here, the remote answers and no susi client is
mid-mission (the updater waits for an idle susi). The hourly timer
(`susi-release-sync.sh --install-timer`, registering `systemd --user`
`susi-release-sync.timer` on Linux or `RunLevel=Limited` Task Scheduler task
`susi-release-sync` on Windows) does it unattended; `susi-tools workflow
check` shows where the installed copy stands and starts the updater on both
Windows and Linux when it is behind. `--check` reports `installed:`,
`qualified:`, and `latest:`. "Remote binary" means the remote release: no
binary is downloaded, the release is built from its tag on the host and
compared with the record's reference checksum. When promoting the exact
reference binary produced by `release-qualify` on the qualifying host after Step
5 lands on `origin`, `susi-release-sync.sh --tag vX.Y.Z --built <path>`
verifies `<path>`'s SHA-256 against `releases/vX.Y.Z.json` and promotes with
health check and rollback without a second cold build.

### What the sha256 means

The `sha256` of an asset is the digest of the **reference build**: the binary
built when the release was cut, with the recorded command, settings and
toolchain, on the recorded host. It identifies that binary. It is not a gate
for anyone else's build: your own build of the same tag may differ in bytes
(paths, linker, toolchain build, host). Installers report whether a build
matches the reference digest, and they never refuse a build for differing.
**The gate is the version check**: the built binary must report the tag's
version, and the tag must point at the record's commit.

## Cutting a release: not available today

Until 2026-11-22 a release was cut with the release tools in the private
repository's `tools/core-xtask`: `release-qualify vX.Y.Z` ran every gate on the
exact tree from a snapshot of main (format, check, Clippy, licences, the whole
test suite, the reference build, smoke, startup and conformance), wrote
`releases/qualification/<tag>.json` only when all passed, and created the
annotated tag; its `--verify-only` ran the checks before the gates and wrote
nothing; and the pre-push `push-policy` admitted a `vX.Y.Z` tag only on a tree
whose record showed every gate at exit 0. Those tools left with
`tools/core-xtask` when the private repository was cut to its licence and vault
core on 2026-11-22. They are to be rebuilt in Susi Tools. Until then no tool cuts
or qualifies a Core release, and no new `vX.Y.Z` tag is published; the tags and
records already published stay valid.

After a release qualified for the current host was published, the installed
copy and the live daemon were synced to it with
`scripts/susi-release-sync.sh --tag vX.Y.Z` (see below).

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

Core standalone installation and upgrades target native `x86_64` Linux
(`install.ps1` stays in this repository only as legacy migration debt). Both
active Linux paths resolve the newest tag with `git ls-remote --tags` against a
configured zero-GitHub canonical or mirror Git remote, check that tag out, read
its record from the remote's default branch, build `susi` natively with the
recorded profile, features and environment, and install the result only when it
reports the tag's version.

They need git and a Rust toolchain installed through rustup (plus a local or
self-hosted Cargo crate mirror when building completely offline). The tag's
`rust-toolchain.toml` pins the compiler (for example, 1.99.0); when that
toolchain is not installed the installers stop and print the
`rustup toolchain install` command. They never install system software.

| Path | Remote |
|---|---|
| `install.sh` (Linux x86_64) | `SUSI_SOURCE_REPO`, else the `origin` of the checkout the script runs from |
| `scripts/susi-release-sync.sh` (Linux x86_64 upgrades; installs its own next copy from the default branch, not from the tag) | `SUSI_RELEASE_REMOTE`, else the `origin` of the private release checkout `~/.susi/release-src` that the installer or an earlier sync made |
| `install.ps1` (legacy Windows migration debt) | `SUSI_SOURCE_REPO`, else the `origin` of the checkout the script runs from |

`SUSI_RELEASE_TAG` (installers) and `--tag` (sync) pin a specific release.
There is no built-in default remote: a script that finds none says which
variable to set.

## Commercial certification

Susi's `scripts/refresh-certified-core.sh` fetches the core
repository over git (its configured core remote) and takes the pinned
release's commit and reference digests from `releases/<tag>.json`.
