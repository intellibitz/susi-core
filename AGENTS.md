# Susi Core: the contract

Susi Core is the small set of libraries that Susi, the product, builds on. This page is the contract every agent reads before
editing here. Its requirements are rules, not claims of completed implementation; what is not
finished is said at the end. Agents own authorized development, verification, integration and
production qualification end-to-end; do not ask the requester to inspect code or perform
routine engineering steps.

## What this repository is

- **Three repositories, one ecosystem.** Susi is the product: a private, licensed set of
  commercial applications, and the primary. Susi Core (this repository, `susi-core`) and
  Susi Tools (`susi-tools`, everything used only while developing, testing or releasing) are
  its public Apache-2.0 derivatives. The three move together. The root package here is still
  called `susi`; it holds no program.
- **Dependencies flow one way**: from Susi to Susi Core and Susi Tools, never back, and Susi
  Core and Susi Tools do not depend on each other. Core builds and tests on its own.
- **Nothing secret and nothing of the private product enters this repository**: no
  credential, key, licence material, private record or proprietary source, in any file or in
  the history.
- **Development-only code never ships with production code.** What is used only while
  developing, testing or releasing belongs in Susi Tools. Tests stay beside the code they
  test.
- **What it holds:** the libraries Susi builds on, each named in
  [`ARCHITECTURE.md`](ARCHITECTURE.md) with what Susi uses it for. There is no program: the
  earlier one (an MCP server and a local daemon) left on 2026-11-22 because no feature of Susi
  used it; v0.21.72 is its last release.
- **Agents land features without being asked.** Pick up the open task or request that
  unblocks the most and put it where the next section says.
- **Nothing is a permanent ceiling, and nothing has a line limit.** A rule that stops serving
  its reason is corrected in place, saying why.

## Where a feature goes

Core holds only what Susi's features use (decided on 2026-11-22). A library capability enters
when a feature of Susi needs it, it is useful without a licence, it is fresh code written here
under this license and it passes the Linux gates and the offline test with every mirror cut
off. Anything paid for, or touching licensing, a vault or anything secret, goes to Susi;
anything used only while developing, testing or releasing goes to Susi Tools. A library no
feature of Susi uses leaves, after Susi has stopped naming it.

## The libraries and their size

- Every package is one Susi builds on, and every package and third-party crate is named, with
  what Susi uses it for, in [`ARCHITECTURE.md`](ARCHITECTURE.md); the tests fail otherwise.
- Removing what Susi still uses breaks Susi: Susi stops using it first. No functionality is
  dropped, no error stub replaces a working capability, and no implementation is merely
  relabeled to reduce a count. Keep proprietary source out of core. Core ships no UI or
  bundled plug content.
- Core has no line limit. Size is measured and reported, never refused: `cargo core-size`
  prints the libraries' physical Rust lines under the counting contract in
  `docs/lean-core.md`, and the other counts pinned in `tests/lean_core_ceilings.rs` are
  recorded baselines that a change may raise in the same commit, saying why. Never hide or
  minify code, or move it into an excluded category, to change a reported number.
- Keep library code and current documentation generic: no personal, corporate or individual
  identity references. Preserve mandatory license and attribution notices.

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
   report, offline (natively on Linux; from a Windows host, in WSL2). No hook enforces them;
   every landing runs them.
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
  copies of this repository receive only `main` and signed release tags; an unsigned tag is
  never published. Nothing is ever fetched from a hosted copy: clone, build and test must
  succeed with every mirror and the internet unreachable. Use no hosting-service API, hosted CI or hosted review.
- Only `refs/heads/main` is published as a branch. Agent handoffs use `refs/agents/<agent>`.
  Release tags are immutable. Never force-push or change live access controls.

## Speed

- Every operation a developer or agent waits on here (build, test, lint, the Linux gates,
  commit) finishes within 10 seconds warm. Cold timings (empty caches) are measured
  and reported apart, and every timing is quoted with its date, machine and commit.
- Over the line is a defect, recorded in a task with its measured cause and fixed; it is
  never met by skipping, loosening or narrowing a test.
- Run the suite with `cargo nextest run --workspace --offline --locked` (settings in
  `.config/nextest.toml`) and the doc-tests beside it with `cargo test --doc --workspace
  --offline --locked` (core has none today).

## Code, commits and tests

- Library code, build tooling and dependencies are Rust. External crates are owned by their
  `susi-vendor-*` facade; ordinary `susi-*` crates declare no external dependency. Keep the
  leaf dependency order `paths → error`.
- Preserve `forbid(unsafe_code)` and `deny(unsafe_code)` boundaries: no library here has
  unsafe code. Production code has no panic-path macros. Justify every lint allowance inline.
  Return a typed `Result` from every fallible operation, I/O above all. No stubs, fake
  success, dead public interfaces or lowered acceptance baselines.
- Before publishing, run rustfmt and Clippy with `-D warnings` on every crate a change
  affects. Remove what nothing uses in the change that leaves it unused. A marker comment
  names its task, and commented-out code is deleted.
- Each commit is one whole change that builds and passes what it affects, with its trailers
  together in the final paragraph. Its subject says what in at most 72 characters, with no
  trailing period; a change over 100 lines has a body that says why. Commit only source: no
  build output, scratch, local state, blob of 1 MiB or more, or secret; rotate a committed
  secret at once. Never rewrite published history; correct a published mistake with a new
  commit. The hooks in `.githooks/` (enable them once per clone: `git config core.hooksPath
  .githooks`) hand each stage to `susi-tools hook`, from Susi Tools, where their rules live:
  they refuse large blobs, secret-shaped lines, an added `dbg!`, unformatted Rust, a commit
  outside a linked worktree or straight on `main`, and a commit whose task is neither recorded
  nor claimed here or whose claim does not cover it. The public boundary (nothing of the
  private product) is refused by the canonical server at push.
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

- Core is built and tested on x86_64 Linux. Keep the libraries' source portable: Susi builds
  them on Windows too.
- Copied or harvested source must offer pure Apache-2.0, with that alternative explicitly
  elected for dual licensing. Preserve section 4 notices, licenses, modification attribution
  and pinned source provenance. Never relabel third-party code.
- Distributed dependencies permit Apache-2.0, MIT, BSD-2-Clause and BSD-3-Clause under the
  deny policy in `deny.toml`. The one exception, decided on 2026-11-22, is Unicode-3.0 for
  `unicode-ident`, which only the procedural macros use while compiling, so its tables are not
  linked into anything built here; any other exception is a new decision. Audit transitive
  dependencies, native objects, build helpers and the statically linked standard library, not
  only their declared licenses.

## Secrets

Use only the authorized automation vault or scoped process environment. Never open a private
personal vault, invent a secret, print a value or place it in source, logs, chat, task records
or command-line arguments. Inject only a child's required scope, through pipes or its
environment. Never rotate an authority secret or alter credential access as routine cleanup.

## Releases

Since 2026-11-22 Core has no program and cuts no releases: Susi builds on a pinned commit of
`main`. The tags already published stay as they are: Cut each version once and never move,
re-cut or delete its tag.

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

- **Speed:** natively on Linux the gates (formatting, Clippy, the whole suite and the size
  report) take about 5 seconds together, the suite under 2 (2026-11-22, a 28-thread host on
  mains power); from a Windows host, starting WSL2 adds about 12 seconds when it is idle.
- **What Susi uses is thin:** Susi's vault uses the four facades; `susi-paths` and
  `susi-error` are reached only by Susi's Windows crate, and `susi-error` only by one of its
  tests. Whether they stay is decided with a check on a Windows host.
- **The public descriptions** of the hosted copies still describe the earlier program until
  they are changed there, which is an external publication.
- **Development code:** none is left here. The hooks and scripts that ran only for the
  earlier program, and the earlier runtime's roadmap, verdicts, evidence ledger, lanes and
  baselines, left on 2026-11-22; the hooks' rules and the scripts they ran moved to Susi Tools
  on 2026-11-23, leaving one-line hooks.
- **Old records:** the goal notes in `.agents/visions/carried/` name sections of the earlier
  contract by number, and the rules they name are kept here word for word.
- **Older pins:** counts pinned in `tests/architecture_tests.rs` are still worded as ceilings
  that only go down.
