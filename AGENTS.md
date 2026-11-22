# Susi Core: the contract

Susi Core is a generic standards-protocol AI OS socket: a small Rust runtime that serves
standard protocols on your own machine. This page is the contract every agent reads before
editing here. Its requirements are rules, not claims of completed implementation; what is not
finished is said at the end. Agents own authorized development, verification, integration and
production qualification end-to-end; do not ask the requester to inspect code or perform
routine engineering steps.

## What this repository is

- **Three repositories, one ecosystem.** Susi is the product: a private, licensed set of
  commercial applications, and the primary. Susi Core (this repository, `susi-core`) and
  Susi Tools (`susi-tools`, everything used only while developing, testing or releasing) are
  its public Apache-2.0 derivatives. The three move together. The program built here is
  still called `susi`; it becomes `susi-core` in a later, announced step.
- **Dependencies flow one way**: from Susi to Susi Core and Susi Tools, never back, and Susi
  Core and Susi Tools do not depend on each other. Core builds and tests on its own.
- **Nothing secret and nothing of the private product enters this repository**: no
  credential, key, licence material, private record or proprietary source, in any file or in
  the history.
- **Development-only code never ships with production code.** What is used only while
  developing, testing or releasing belongs in Susi Tools. Tests stay beside the code they
  test.
- **What runs today:** `susi mcp serve` (an MCP server over standard input and output) and
  the local daemon (`susi start`, `susi status`, `susi stop`); the README says how to use
  them.
- **Agents land features without being asked.** Pick up the open task or request that
  unblocks the most and put it where the next section says.
- **Nothing is a permanent ceiling, and nothing has a line limit.** A rule that stops serving
  its reason is corrected in place, saying why.

## Where a feature goes

Core improves on a weekly cycle: a change proven in dependent work enters only when it helps a
single developer on its own, is not paid-only value, is fresh code written here under this
license and passes the Linux gates and the offline test with every mirror cut off. Anything
paid for, or touching licensing, a vault or anything secret, goes to Susi; anything used only
while developing, testing or releasing goes to Susi Tools. A cycle that lands anything ships a
signed release with plain notes.

## The socket and its size

- Keep three core pillars: **GAWD** (agents), **GEMI** (models) and **GMCP** (tools: the Model
  Context Protocol). A pillar holds code when a program serves it: today GMCP serves, and GAWD
  and GEMI hold none. Preserve the public capabilities, wire contracts, errors and
  compatibility of what the program serves as core changes. Until 2026-11-22 this rule also
  kept GAWD's Agent2Agent and Agent Client Protocol wire contracts, which no program served or
  read; they left that day with the packages that held them, after the compiler confirmed
  nothing used them, because code no program runs is a promise nobody keeps and it made every
  build and check slower. They remain in the published v0.21.72.
- Every package is used by the program or by Susi, and every package and third-party crate is
  named, with its reason, in [`ARCHITECTURE.md`](ARCHITECTURE.md); the tests fail otherwise.
- A host extends the MCP server with its own tools through `Server::with_tool`, without
  editing core.
- Core contains generic protocol contracts and socket machinery. Concrete implementations
  belong in separately hosted implementation packages, including Susi. Prove the destination
  works through the standard socket before removing its implementation from core. No
  functionality is dropped, no error stub replaces a working capability, and no
  implementation is merely relabeled to reduce a count. Keep proprietary source out of core.
  Core ships no UI or bundled plug content.
- Core has no line limit. Size is measured and reported, never refused: `cargo core-size`
  prints the socket's physical runtime Rust lines under the counting contract in
  `docs/lean-core.md`, and the other counts pinned in `tests/lean_core_ceilings.rs` are
  recorded baselines that a change may raise in the same commit, saying why. Never hide or
  minify code, or move it into an excluded category, to change a reported number.
- Keep runtime and current documentation generic: no personal, corporate or individual
  identity references. Preserve protocol identifiers and mandatory license and attribution
  notices.

## Working here

1. Work in your own agent root, worktree, branch and identity, with your own `SUSI_HOME`,
   `CARGO_TARGET_DIR` and `SUSI_PORT_OFFSET`. Never edit the primary checkout, another
   agent's files or the requester's working directory.
2. Start every session with `susi-tools workflow check`: it fast-forwards a clean checkout,
   shows what is addressed to you and says what you hold.
3. Claim before your first edit. A new task takes one command, `susi-tools tasks start
   "<title>" --accept "<cargo command>" --scope <path>`, which adds its record, commits and
   ships it, and claims it; an open task takes `susi-tools tasks claim <id> --scope <path>`.
   Keep one live claim per worker, and when you stop, hand off or finish, release it
   in the same step. Preserve unpublished work or deliberately abandon it with explicit
   authority; never silently discard it.
4. Commit with a `Task: <id>` trailer. Push the commit to `refs/agents/<your-name>` and run
   the Linux gates on that exact commit: formatting, Clippy, the whole suite and the size
   report, offline (from a Windows host, in WSL2). No hook enforces them; every landing runs
   them.
5. Ship with `susi-tools ship`, a fast-forward of `main`; merge first when `main` has moved
   on. Verify the remote contains the commit before claiming it landed.
6. Close with `susi-tools tasks finish <id>`, which runs the task's acceptance, commits the
   close and ships it (`tasks close` then `ship`): a reservation
   or successful compile is not acceptance. Close only after the entire acceptance check
   passes on the actual combined tree.
7. Remove your finished worktrees once every commit is on `origin/main`, the tree is clean
   and no live claim or running agent owns them. Never remove the primary checkout, another
   agent's work, unpublished commits or shared build caches.

Agents may commit, push and merge authorized work without routine approval. This scope does
not authorize unrelated spending, destructive operations or external publication. Send
external messages only with explicit user or invoked-workflow authorization.

## Publication and Git

- Core is Apache-2.0 and published one way. The designated publisher alone holds push
  authority. Authorized agents may exercise that authority within their scope; this policy
  grants no additional write access. Accept no outside pull request, patch or commit.
  Everyone is free to fork under the license; the license grants code rights, not rights to
  the Susi name or marks.
- A local canonical Git server is the source of truth and the `origin` of every clone. GitHub
  and GitLab hold write-only backup copies, refreshed every ten minutes, and the public
  copies of this repository receive only `main` and signed release tags. Nothing is ever
  fetched from a hosted copy: clone, build, test and release must succeed with every mirror
  and the internet unreachable. Use no hosting-service API, hosted CI or hosted review.
- Only `refs/heads/main` is published as a branch. Agent handoffs use `refs/agents/<agent>`.
  Release tags are immutable. Never force-push or change live access controls.

## Speed

- Every operation a developer or agent waits on here (build, test, lint, the Linux gates,
  commit, release) finishes within 10 seconds warm. Cold timings (empty caches) are measured
  and reported apart, and every timing is quoted with its date, machine and commit.
- Over the line is a defect, recorded in a task with its measured cause and fixed; it is
  never met by skipping, loosening or narrowing a test.
- Run the suite with `cargo nextest run --workspace --offline --locked` (settings in
  `.config/nextest.toml`) and the doc-tests beside it with `cargo test --doc --workspace
  --offline --locked` (core has none today).

## Code, commits and tests

- Runtime, build, install and release tooling and shipped dependencies are Rust. External
  crates are owned by their `susi-vendor-*` facade; ordinary `susi-*` crates declare no
  external dependency. Keep the leaf dependency order `paths → error → config → core/sandbox
  → services → daemon`.
- Preserve `forbid(unsafe_code)` and `deny(unsafe_code)` boundaries; the existing platform
  FFI exceptions stay scoped per function with `// SAFETY:` reasoning. Production code has no
  panic-path macros. Justify every lint allowance inline. Return typed `Result<T, EaiError>`
  from I/O, state and IPC. Check HTTP status explicitly. No stubs, fake success, dead public
  interfaces or lowered acceptance baselines.
- Before publishing, run rustfmt and Clippy with `-D warnings` on every crate a change
  affects. Remove what nothing uses in the change that leaves it unused. A marker comment
  names its task, and commented-out code is deleted.
- Each commit is one whole change that builds and passes what it affects, with its trailers
  together in the final paragraph. Its subject says what in at most 72 characters, with no
  trailing period; a change over 100 lines has a body that says why. Commit only source: no
  build output, scratch, local state, blob of 1 MiB or more, or secret; rotate a committed
  secret at once. Never rewrite published history; correct a published mistake with a new
  commit. The hooks in `.githooks/` refuse large blobs, secret-shaped lines and an added
  `dbg!`.
- Tests are hermetic: private HOME, USERPROFILE, XDG and SUSI_HOME, scratch configuration,
  ports and local Git remotes; no production state or inherited credentials. Do not send real
  coordination messages from tests. Never skip, ignore, loosen or delete a test to get green.
  Each test proves one named behavior. Pin a count or phrase only when it is the contract,
  and move the pin in the change that moves it. Remove tests and fixtures with the code they
  cover. Nonempty executed tests are required; ignored tests and compilation are not passes.
- Use the pinned compiler and committed lockfile. The required checks:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --offline --locked -- -D warnings
cargo nextest run --workspace --offline --locked
cargo deny --offline --locked check licenses
```

## Platform and licensing

- Core ships for x86_64 Linux only; qualify releases only on a native x86_64 Linux host.
  Keep portable source for other targets without adding their builds, tests or releases.
- Copied or harvested source must offer pure Apache-2.0, with that alternative explicitly
  elected for dual licensing. Preserve section 4 notices, licenses, modification attribution
  and pinned source provenance. Never relabel third-party code.
- Distributed dependencies permit Apache-2.0, MIT, BSD-2-Clause and BSD-3-Clause under the
  deny policy in `deny.toml`. The one exception, decided on 2026-11-22, is Unicode-3.0 for
  `unicode-ident`, which only the procedural macros use while compiling, so its tables are not
  linked into the program; any other exception is a new decision. Audit transitive
  dependencies, native objects, build helpers and the statically linked standard library, not
  only their declared licenses.

## Secrets

Use only the authorized automation vault or scoped process environment. Never open a private
personal vault, invent a secret, print a value or place it in source, logs, chat, task records
or command-line arguments. Inject only a child's required scope, through pipes or its
environment. Never rotate an authority secret or alter credential access as routine cleanup.

## Releases

A release is a signed tag `vX.Y.Z` on `main`, cut with `susi-tools release qualify --task <id>
--sign` on a native x86_64 Linux host and published with `susi-tools release publish vX.Y.Z`;
an unsigned tag is never published. Cut each
version once and never move, re-cut or delete its tag; follow a broken release with a new
version. Publish the tag, its record and its qualification record together. Never install an
unqualified candidate as production. Use atomic replacement, health checks and tested rollback
to the previous verified artifact; `susi install` never replaces a different existing program.

## Records and documents

- Operational history, completed tasks, evidence and audit records belong outside checked-out
  source. Missing history never proves acceptance, a completed dependency or a qualified
  release.
- Documentation reflects reality at all times: a document says what is true now and changes
  in the change that alters what it describes, and is corrected or removed when its subject
  goes. One topic lives in one document, listed in `docs/README.md`.
- State each rule once, here, and point to it elsewhere instead of copying it. Agent rule
  files stay identical pointers that restate nothing (`CLAUDE.md`, `GEMINI.md`).

## Not finished

- **Speed:** a warm repeat of the Linux gates takes 6.9 seconds, the tests 5 (2026-11-22, a
  28-core host through WSL2, after the packages no program used left); the first run after a
  change is still over the line (39 seconds of steps after that change, 29 of them compiling,
  linking and running 54 test binaries).
- **Releases:** `susi-tools release` cut `v0.21.72`, the first release of today's program
  (`docs/releases.md`). The unsafe-code check and the standard-library builder's runner left
  with Susi's `tools/core-xtask` on 2026-11-22 and are still to be rebuilt.
- **Development code still here:** `scripts/` and `toolchain/` belong in Susi Tools.
- **The pillars:** GMCP serves (`susi mcp serve`); GAWD and GEMI hold no code yet.
- **Old records:** `config/` holds the earlier runtime's catalogs (models, agents, vendors),
  and `.agents/roadmap.json` its roadmap; the goal notes in `.agents/visions/carried/`
  name sections of the earlier contract by number, and the rules they name are kept here
  word for word.
- **Older pins:** counts pinned in `tests/architecture_tests.rs` and `tests/socket_boundary.rs`
  are still worded as ceilings that only go down.
