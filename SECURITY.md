# Security Policy

## Supported Versions

Susi Core has no program and cuts no releases since 2026-11-22: security fixes land on `main`,
which Susi builds on at a pinned commit. The last release with a program, `v0.21.72`, is no
longer maintained.

| Version | Supported |
| ------- | --------- |
| `main` | :white_check_mark: |
| Release tags (`v0.21.72` and earlier) | :x: |

## What the libraries touch

Read from `crates/` on 2026-11-22. No library opens a network connection, listens on a port or
starts another program.

- **`susi-paths`** resolves the Susi home and the standard folders from `SUSI_HOME`, `HOME` and
  the XDG settings, and creates owner-only directories (mode 0700) when asked. Its test helpers
  isolate a test's home and scrub those variables.
- **`susi-error`** is a metrics sink: it appends the JSON line its caller gives it to the file
  its caller names, creating the file owner-only, and rotates it once past 64 MiB.
- **The four `susi-vendor-*` facades** only re-export serde, serde_json, sha2 and getrandom.

**Limits, said plainly.** The sink masks nothing: what its caller writes reaches the file as
given. Anyone who can read your files can read that file.

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report suspected vulnerabilities privately via [GitHub Private Vulnerability Reporting](https://github.com/intellibitz/susi-core/security/advisories/new) on `intellibitz/susi-core`.

Please include:
- A clear description of the vulnerability and the affected library (e.g., `susi-paths`,
  `susi-error`).
- Minimal reproduction steps or a proof-of-concept test case.
- Potential impact (e.g., a metrics file created with wider permissions than stated above).

We aim to acknowledge vulnerability reports within **48 hours** and publish a verified fix and regression test as quickly as possible.
