# Susi Core vision

Susi Core exists for one reason: to hold, in the open, the small libraries that Susi, the
product, builds on. Susi is what is sold; Susi Core and Susi Tools are its public Apache-2.0
derivatives, and every line here must trace to a feature of Susi that uses it (decided on
2026-11-22).

What that means in practice:

- **Only what Susi uses.** A library enters when a feature of Susi needs it and it is useful
  without a licence: home and path resolution, a metrics sink, the wrappers of the third-party
  crates Susi uses. A library no feature of Susi uses leaves. [`ARCHITECTURE.md`](ARCHITECTURE.md)
  names each one with what Susi uses it for, and the tests fail on anything else.
- **No program.** Until 2026-11-22 Core had one, a Model Context Protocol server with
  read-only file tools and a local daemon. No feature of Susi used it, so it left; its last
  release, v0.21.72, stays published.
- **Small and fast.** No line limit, but every size is measured and reported, and nothing a
  developer or agent waits on takes more than 10 seconds warm.
- **Nothing secret, nothing sold.** No credential, licence material or proprietary source, in
  any file or in the history. Paid value lives in Susi.

Core is published one way: no outside pull request, patch or commit is merged. Everyone is
free to fork under the license, which grants code rights, not rights to the Susi name or marks.
