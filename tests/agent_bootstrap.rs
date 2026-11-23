#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]
#![allow(missing_docs)] // integration test crate: no public API to document

//! Agent directives stay pointers to the contract: every tool's rule file names
//! `AGENTS.md` and restates nothing, and no tool runs a startup hook before it
//! is read (AGENTS.md, "Records and documents").

use std::path::Path;
use std::process::Command;

#[test]
fn every_agent_tool_auto_loads_the_same_first_step() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let files = ["CLAUDE.md", "GEMINI.md"];
    let canonical = std::fs::read_to_string(root.join(files[0])).unwrap();
    for file in files {
        let text = std::fs::read_to_string(root.join(file)).unwrap();
        assert_eq!(
            text, canonical,
            "{file} diverged from the canonical pointer"
        );
        for required in [
            "AGENTS.md",
            "canonical active engineering",
            "Agents own authorized development",
        ] {
            assert!(text.contains(required), "{file} must carry {required}");
        }
    }
    // No tool may silently run a startup hook before reading the contract.
    // Useful per-tool permissions remain intact.
    for hook_file in [".claude/settings.json", ".codex/hooks.json"] {
        if let Ok(text) = std::fs::read_to_string(root.join(hook_file)) {
            let settings: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert!(
                settings
                    .get("hooks")
                    .is_none_or(|h| h.as_object().is_some_and(|h| h.is_empty())),
                "{hook_file} contains unreviewed automatic startup hooks"
            );
        }
    }
}

/// Discover tracked tool directives so adding a new agent cannot quietly bring
/// in a divergent policy document: each Markdown directive points at the contract.
#[test]
fn tracked_agent_directives_point_at_the_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let listed = Command::new("git")
        .current_dir(root)
        .args(["ls-files", "-z"])
        .output()
        .unwrap();
    assert!(listed.status.success());
    let mut inspected = 0;
    for bytes in listed.stdout.split(|b| *b == 0).filter(|b| !b.is_empty()) {
        let path = std::str::from_utf8(bytes).unwrap();
        let tool_directory = [
            ".claude/",
            ".codex/",
            ".cursor/",
            ".gemini/",
            ".windsurf/",
            ".roo/",
            ".kilocode/",
            ".kiro/",
            ".junie/",
            ".opencode/",
            ".openhands/",
            ".continue/",
            ".github/instructions/",
        ]
        .iter()
        .any(|prefix| path.starts_with(prefix));
        let tool_file = [
            "CLAUDE.md",
            "GEMINI.md",
            ".github/copilot-instructions.md",
            ".cursorrules",
            ".windsurfrules",
            ".clinerules",
            ".aider.conf.yml",
        ]
        .contains(&path);
        if !tool_directory && !tool_file {
            continue;
        }
        // A deleted tracked file is absent before the commit; its removal is
        // intentional, not a reason to resurrect it.
        let Ok(text) = std::fs::read_to_string(root.join(path)) else {
            assert!(
                !root.join(path).exists(),
                "unreadable tracked directive {path}"
            );
            continue;
        };
        inspected += 1;
        if path.ends_with(".md") || path.ends_with(".mdc") || tool_file {
            assert!(
                text.contains("AGENTS.md"),
                "{path} must reference the canonical contract"
            );
        }
    }
    // CLAUDE.md, GEMINI.md and the tracked tool directories; the Copilot file
    // left with the .github/ directory (T-CLAUDENOGITHUB-1).
    assert!(
        inspected >= 3,
        "the known agent entrypoints must be inspected"
    );
}
