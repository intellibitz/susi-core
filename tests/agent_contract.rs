#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! The rules Susi Core's contract, `AGENTS.md`, keeps in its own words.
//!
//! The contract is the page every agent reads first, so it stays short and true. A phrase is
//! pinned here only because it is the rule itself ("Pin a count or phrase only when it is the
//! contract"); wording that described a retired process is not pinned. A rule that changes
//! moves its phrase here in the same change.

use std::path::Path;

/// A tracked file with its whitespace folded, so a rule may wrap anywhere on the page.
fn flat(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {rel}: {error}"))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

const RULES: &[&str] = &[
    // What Core is.
    "generic standards-protocol AI OS socket",
    "not claims of completed implementation",
    "Agents own authorized development",
    "Dependencies flow one way",
    "Nothing secret and nothing of the private product enters this repository",
    "Development-only code never ships with production code",
    "Tests stay beside the code they test",
    "Agents land features without being asked",
    "Nothing is a permanent ceiling, and nothing has a line limit",
    "Core improves on a weekly cycle",
    // The socket and its size.
    "exactly three core pillars",
    "Concrete implementations belong in separately hosted implementation packages",
    "Prove the destination works through the standard socket before removing",
    "No functionality is dropped",
    "Keep proprietary source out of core",
    "Core ships no UI or bundled plug content",
    "Core has no line limit",
    "Size is measured and reported, never refused",
    "Never hide or minify code",
    "Keep runtime and current documentation generic",
    // Publication, Git and the hosted copies.
    "published one way",
    "The designated publisher alone holds push authority",
    "Authorized agents may exercise that authority within their scope",
    "this policy grants no additional write access",
    "Accept no outside pull request, patch or commit",
    "Everyone is free to fork under the license",
    "the license grants code rights, not rights to the Susi name or marks",
    "an unsigned tag is never published",
    "Nothing is ever fetched from a hosted copy",
    "receive only `main` and signed release tags",
    "clone, build, test and release must succeed with every mirror and the internet unreachable",
    "Only `refs/heads/main` is published as a branch",
    "Agent handoffs use `refs/agents/<agent>`",
    "Release tags are immutable",
    "Never force-push or change live access controls",
    "Verify the remote contains the commit before claiming it landed",
    "Agents may commit, push and merge authorized work without routine approval",
    "This scope does not authorize unrelated spending, destructive operations or external publication",
    "Send external messages only with explicit user or invoked-workflow authorization",
    // Claims, acceptance and records.
    "one live claim per worker",
    "release it in the same step",
    "Preserve unpublished work",
    "a reservation or successful compile is not acceptance",
    "Close only after the entire acceptance check passes on the actual combined tree",
    "Never remove the primary checkout",
    "Missing history never proves acceptance",
    "Documentation reflects reality at all times",
    "State each rule once",
    "Agent rule files stay identical pointers that restate nothing",
    // Platform and licensing.
    "Core ships for x86_64 Linux only",
    "qualify releases only on a native x86_64 Linux host",
    "pure Apache-2.0",
    "Preserve section 4 notices",
    "Never relabel third-party code",
    "Apache-2.0, MIT, BSD-2-Clause and BSD-3-Clause",
    // Code, commits and tests.
    "Preserve `forbid(unsafe_code)` and `deny(unsafe_code)` boundaries",
    "Production code has no panic-path macros",
    "Justify every lint allowance inline",
    "run rustfmt and Clippy with `-D warnings` on every crate a change affects",
    "Remove what nothing uses in the change that leaves it unused",
    "typed `Result<T, EaiError>`",
    "owned by their `susi-vendor-*` facade",
    "Check HTTP status explicitly",
    "No stubs, fake success",
    "Its subject says what in at most 72 characters, with no trailing period",
    "Commit only source",
    "rotate a committed secret at once",
    "Never rewrite published history; correct a published mistake with a new commit",
    "Tests are hermetic",
    "Do not send real coordination messages from tests",
    "Never skip, ignore, loosen or delete a test to get green",
    "Pin a count or phrase only when it is the contract",
    "Remove tests and fixtures with the code they cover",
    "Nonempty executed tests are required",
    "Use the pinned compiler and committed lockfile",
    "cargo deny --offline --locked check licenses",
    // Speed.
    "finishes within 10 seconds warm",
    "Cold timings (empty caches) are measured and reported apart",
    "never met by skipping, loosening or narrowing a test",
    "cargo nextest run --workspace --offline --locked",
    // Secrets.
    "only the authorized automation vault or scoped process environment",
    "Never open a private personal vault",
    "Inject only a child's required scope",
    "Never rotate an authority secret or alter credential access as routine cleanup",
    // Releases and rollback.
    "Never install an unqualified candidate as production",
    "Use atomic replacement, health checks and tested rollback",
    "Cut each version once and never move, re-cut or delete its tag",
    "follow a broken release with a new version",
];

#[test]
fn the_contract_keeps_its_rules_in_their_words() {
    let contract = flat("AGENTS.md");
    let missing: Vec<_> = RULES
        .iter()
        .filter(|rule| !contract.contains(**rule))
        .collect();
    assert!(
        missing.is_empty(),
        "AGENTS.md lost these rules: {missing:#?}"
    );
}

/// The agent pointers promise that the contract covers publication, licensing, acceptance and
/// rollback; each promise has its rule on the page.
#[test]
fn the_agent_pointers_promise_only_what_the_contract_holds() {
    let contract = flat("AGENTS.md");
    for pointer in ["CLAUDE.md", "GEMINI.md"] {
        assert!(
            flat(pointer)
                .contains("Follow AGENTS.md for publication, licensing, acceptance and rollback"),
            "{pointer} must point at the contract for its four topics"
        );
    }
    for (topic, rule) in [
        ("publication", "published one way"),
        ("licensing", "pure Apache-2.0"),
        ("acceptance", "the entire acceptance check passes"),
        ("rollback", "tested rollback"),
    ] {
        assert!(contract.contains(rule), "AGENTS.md has no {topic} rule");
    }
}

/// Core is public: its contract and the pages that point to it name no person, company or
/// private record.
#[test]
fn the_contract_and_its_pointers_stay_generic() {
    for path in ["AGENTS.md", "README.md", "CLAUDE.md", "GEMINI.md"] {
        let text = flat(path);
        for reference in [
            "CEO",
            "intellibitz",
            "IntelliBitz",
            "Mandate 8",
            "dot, opus",
        ] {
            assert!(
                !text.contains(reference),
                "{path}: identity or history returned: {reference}"
            );
        }
    }
}
