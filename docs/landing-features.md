# Landing a feature in Susi Core

One page an agent can act from alone. The rules are in the
[active contract](../AGENTS.md); this page is the route through them.

## Why

Susi is the product: private, licensed commercial applications, built by agents
and sold. Susi Core (this repository) and Susi Tools are its public Apache-2.0
derivatives. They hold no secrets, anyone may fork them, and they are how the
work earns its name in the open. A feature that lands in the right one of the
three makes the product better or the public work more useful, so nobody has
to ask for it first.

## What to work on, unprompted

Take the first that applies:

1. A request or a stop addressed to you on the Susi Board
   (`susi-tools workflow board check`).
2. A recorded vision no task serves (`susi-tools workflow vision cycle`).
3. An open task nobody holds (`susi-tools tasks list`).
4. The weekly cycle that feeds this repository: something Susi has proven that
   a single developer would want here (contract rule 7).

## Where it goes

| The feature is | It belongs in |
|---|---|
| paid for, or secret in any part | Susi (private) |
| used only while developing, testing or releasing | Susi Tools |
| a small runtime capability, useful to one developer with no licence, written fresh here and passing the Linux gates | Susi Core |
| anything else | Susi |

Size is never a condition: no repository has a line limit.

## How to land it here

```sh
susi-tools workflow start <name>              # your own worktree and branch
susi-tools workflow check                     # first command in it
susi-tools workflow board check               # what stops you, what waits for you
susi-tools tasks add "<title>" --accept "<command>" --goal "<why>"
susi-tools tasks claim <id> --scope <path>    # before the first edit; one hour
susi-tools tasks renew <id>                   # while the work goes on
susi-tools workflow commit -m "<what and why>" --task <id>
susi-tools workflow ship                      # merge origin/main, land on main
susi-tools workflow finish <id>               # when done: verify, close, release
```

Publish with `ship` whenever a step is meaningful. `finish` ends the task: it
closes only when the task's acceptance passes. Each verb's `--help` is the
authority on what it does. A task that is not small also records where you
looked for existing work first (`susi-tools workflow reuse --help`).

Before work reaches `main`, pass the Linux gates. Susi Core builds on x86_64
Linux only, so its acceptance runs there too:

```sh
cargo build --workspace --offline --locked
cargo test --workspace --offline --locked
cargo core-size                               # reports the size; never fails on it
cargo clippy --workspace --all-targets --offline --locked -- -D warnings
```

On a Windows host the same four run in WSL2 with the network cut off, through
the gate script on the canonical Git server (`bin/core-linux-gates.ps1 -Agent
<name> -Rev <commit> -Lint`). It tests a commit the server already holds, so
publish your branch tip to `refs/agents/<agent>` first.

## What not to do

- Do not copy code from Susi into this repository; write it fresh here.
- Do not add a secret, a licence check or anything that needs the private
  product to build or run.
- Do not fetch from, push to or depend on a hosted copy (GitHub, GitLab).
- Do not skip, loosen or delete a test to get green, and do not wait to be asked.

## Not finished

The rename of this repository's program from `susi` to `susi-core` (the
repository itself is `susi-core` since 2026-11-22), and the move of
development-only code to Susi Tools, which holds no tooling yet.
The contract's first section lists each open item.
