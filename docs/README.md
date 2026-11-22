# Documentation index

Every document under `docs/` is listed here, with the main documents elsewhere
in the repository. Look here before you add one: extend the document that
already covers your topic, and add a line here for a new one in the same change.
The rules agents work by are in [`AGENTS.md`](../AGENTS.md); they are not
repeated here.

## Start here

- [The contract: how agents work here and where a feature goes](../AGENTS.md)
- [The README: what Susi Core is and what runs today](../README.md)

## What the program is

- [Architecture: what runs, the crates and how they may depend on each other](../ARCHITECTURE.md)
- [Security: what the program exposes and how to report a vulnerability](../SECURITY.md)
- [Integrating with Susi Core](integrating-with-susi.md)
- [Lean core: how the socket's size is measured and reported](lean-core.md)

## Releases

- [Releases: the record format, cutting a release, installing and upgrading](releases.md)

## Direction and the project

- [Vision](../VISION.md)
- [Forking and contributing](../CONTRIBUTING.md)
- [Contributors and credits](../CONTRIBUTORS.md)

## Records

- On 2026-11-22 the pages about features that had left core were removed: the
  harness and its runbooks, the peer channel, extension packs, local Windows
  channels, release distillation and release lanes, gate receipts, the Git
  storage publisher, cloud sessions, the host contract and its conformance
  checker, the Unicode and identifier adapters, the earlier architecture and
  threat model, the evidence page, the step-1 acceptance record and the
  historical workflow measurements. The numbered directive documents
  (`ceo-agent-mandate.md`, `ceo-mandate-74.md` to `ceo-mandate-87.md`) and the
  landing page (`landing-features.md`) left the same day. Their rules that
  still hold are in [`AGENTS.md`](../AGENTS.md); the pages stay in the history
  (the directive documents last in commit `027bab400`, the others in
  `7525cf55b`).
