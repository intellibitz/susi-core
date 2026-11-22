#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]

//! Architecture boundary tests — see `ARCHITECTURE.md`.
//!
//! Parses workspace `Cargo.toml` files and `Cargo.lock` (no network) and asserts:
//! - every package of the workspace and every crate the lock file resolves is named in
//!   `ARCHITECTURE.md`, so nothing enters Core without saying why (an allow-list: until
//!   2026-11-22 this suite kept deny-lists of crates and modules deleted long before);
//! - no workspace crate dependency cycles, and every SUSI edge points down the leaf order;
//! - cross-crate source mounts stay at zero.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn parse_workspace_deps(toml: &str) -> HashSet<String> {
    let mut deps = HashSet::new();
    let mut in_deps = false;
    for line in toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            // Production edges only — ignore [dev-dependencies] / [target.*.dev-dependencies].
            in_deps = trimmed == "[dependencies]"
                || trimmed.starts_with("[dependencies.")
                || (trimmed.starts_with("[target.")
                    && trimmed.contains("dependencies]")
                    && !trimmed.contains("dev-dependencies]"));
            continue;
        }
        if !in_deps {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("susi-") {
            if let Some(name_end) = rest.find([' ', '=', '{']) {
                let name = format!("susi-{}", &rest[..name_end]);
                deps.insert(name);
            }
        }
        if let Some(idx) = trimmed.find("path = \"crates/") {
            let rest = &trimmed[idx + "path = \"crates/".len()..];
            if let Some(end) = rest.find('"') {
                deps.insert(rest[..end].to_string());
            }
        }
        if let Some(idx) = trimmed.find("path = \"../") {
            let rest = &trimmed[idx + "path = \"../".len()..];
            if let Some(end) = rest.find('"') {
                let name = rest[..end].to_string();
                if name.starts_with("susi-") {
                    deps.insert(name);
                }
            }
        }
    }
    deps
}

fn workspace_graph() -> HashMap<String, HashSet<String>> {
    let root = workspace_root();
    let mut graph = HashMap::new();

    let root_toml = std::fs::read_to_string(root.join("Cargo.toml")).expect("root Cargo.toml");
    graph.insert("susi".to_string(), parse_workspace_deps(&root_toml));

    let crates_dir = root.join("crates");
    for entry in std::fs::read_dir(&crates_dir).expect("crates/") {
        let entry = entry.expect("entry");
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("susi-") {
            continue;
        }
        let toml_path = entry.path().join("Cargo.toml");
        if !toml_path.is_file() {
            continue;
        }
        let text = std::fs::read_to_string(&toml_path).expect("crate Cargo.toml");
        graph.insert(name, parse_workspace_deps(&text));
    }
    graph
}

/// AGENTS.md leaf order (`paths → error → config → core/sandbox → services
/// → daemon`) as ranks: a crate may depend only on strictly lower-ranked
/// SUSI crates, so shared code is reached through a Cargo edge in one
/// direction instead of being copied or source-mounted upward.
const LEAF_RANK: &[(&str, u8)] = &[("susi-paths", 0), ("susi-error", 1), ("susi", 3)];

/// Pure re-export facades (`susi-vendor-*` crates whose only dependency is
/// the third-party crate they wrap). They are leaves by construction —
/// they depend on no SUSI crate — and are exempt from the leaf-order check:
/// a rank-0 crate like susi-paths legitimately depends on a facade.
/// The zero-external-deps ratchet below is what actually constrains them.
const VENDOR_FACADES: &[&str] = &[
    "susi-vendor-getrandom",
    "susi-vendor-serde",
    "susi-vendor-serde-json",
    "susi-vendor-sha2",
];

#[test]
fn workspace_edges_follow_the_leaf_order() {
    let rank: HashMap<&str, u8> = LEAF_RANK.iter().copied().collect();
    let facades: HashSet<&str> = VENDOR_FACADES.iter().copied().collect();
    let graph = workspace_graph();
    for (package, dependencies) in &graph {
        if facades.contains(package.as_str()) {
            continue; // leaves: they own a third-party crate, depend on no SUSI crate
        }
        let own = *rank
            .get(package.as_str())
            .unwrap_or_else(|| panic!("{package} has no leaf rank in LEAF_RANK"));
        for dependency in dependencies {
            if facades.contains(dependency.as_str()) {
                continue; // facade edge: permitted from every rank
            }
            let theirs = *rank
                .get(dependency.as_str())
                .unwrap_or_else(|| panic!("{dependency} has no leaf rank in LEAF_RANK"));
            assert!(
                theirs < own,
                "{package} (rank {own}) must not depend on {dependency} (rank {theirs}); \
                 edges point down the leaf order only"
            );
        }
    }
}

/// Mandate: susi-* crates declare zero third-party dependencies — every
/// external crate is owned by exactly one `susi-vendor-*` crate (implementation
/// vendors like wasmer/candle, or pure re-export facades in VENDOR_FACADES).
/// This ratchet scans each non-vendor member's `[dependencies]`,
/// `[build-dependencies]` and `[target.*.dependencies]`/`dev-dependencies`
/// sections and requires every declared package to be a workspace member.
#[test]
fn susi_crates_declare_zero_external_dependencies() {
    let root = workspace_root();
    let root_toml = std::fs::read_to_string(root.join("Cargo.toml")).expect("root Cargo.toml");

    // Workspace member names = root package + every members[] dir.
    let mut members: HashSet<String> = ["susi".to_string()].into();
    let mut in_members = false;
    for line in root_toml.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_members = t == "[workspace]";
        }
        if in_members && t.contains('"') {
            // Members may be several per line: "crates/a", "crates/b", ...
            for entry in t.split('"').skip(1).step_by(2) {
                if let Some(name) = entry.strip_prefix("crates/") {
                    members.insert(name.to_string());
                }
            }
        }
    }

    let mut manifests: Vec<std::path::PathBuf> = vec![root.join("Cargo.toml")];
    for entry in std::fs::read_dir(root.join("crates")).expect("crates/") {
        let entry = entry.expect("entry");
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("susi-vendor-") {
            continue; // vendor crates are where external deps belong
        }
        let p = entry.path().join("Cargo.toml");
        if p.is_file() {
            manifests.push(p);
        }
    }
    for path in manifests {
        let text = std::fs::read_to_string(&path).expect("crate Cargo.toml");
        let mut in_deps = false;
        for line in text.lines() {
            let t = line.trim();
            if t.starts_with('[') {
                in_deps = t == "[dependencies]"
                    || t == "[build-dependencies]"
                    || t == "[dev-dependencies]"
                    || (t.starts_with("[target.") && t.contains("dependencies]"));
                continue;
            }
            if !in_deps || t.is_empty() || t.starts_with('#') {
                continue;
            }
            let Some((key, rhs)) = t.split_once('=') else {
                continue;
            };
            let declared = if let Some(i) = rhs.find("package = \"") {
                let rest = &rhs[i + 10..];
                rest.split('"').next().unwrap_or("").to_string()
            } else {
                key.trim().to_string()
            };
            if declared.is_empty() {
                continue;
            }
            assert!(
                members.contains(&declared),
                "{} declares third-party dependency `{declared}` — susi crates \
                 must declare zero external deps; route it through a \
                 susi-vendor-* crate",
                path.display()
            );
        }
    }
}

fn find_cycle(graph: &HashMap<String, HashSet<String>>) -> Option<Vec<String>> {
    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    let mut stack = Vec::new();

    fn dfs(
        node: &str,
        graph: &HashMap<String, HashSet<String>>,
        visiting: &mut HashSet<String>,
        visited: &mut HashSet<String>,
        stack: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        if visited.contains(node) {
            return None;
        }
        if visiting.contains(node) {
            let idx = stack.iter().position(|n| n == node).unwrap_or(0);
            let mut cycle = stack[idx..].to_vec();
            cycle.push(node.to_string());
            return Some(cycle);
        }
        visiting.insert(node.to_string());
        stack.push(node.to_string());
        if let Some(deps) = graph.get(node) {
            for dep in deps {
                if graph.contains_key(dep) {
                    if let Some(c) = dfs(dep, graph, visiting, visited, stack) {
                        return Some(c);
                    }
                }
            }
        }
        stack.pop();
        visiting.remove(node);
        visited.insert(node.to_string());
        None
    }

    for node in graph.keys() {
        if let Some(c) = dfs(node, graph, &mut visiting, &mut visited, &mut stack) {
            return Some(c);
        }
    }
    None
}

#[test]
fn no_workspace_crate_cycles() {
    let graph = workspace_graph();
    if let Some(cycle) = find_cycle(&graph) {
        panic!("workspace crate cycle: {}", cycle.join(" -> "));
    }
}

/// The names in the first column of the table in the section of `ARCHITECTURE.md` that starts
/// with `heading`: every row `| `name` | ...`.
fn names_in_section(heading: &str) -> HashSet<String> {
    let text = std::fs::read_to_string(workspace_root().join("ARCHITECTURE.md")).unwrap();
    let section = text
        .split(&format!("\n{heading}\n"))
        .nth(1)
        .unwrap_or_else(|| panic!("ARCHITECTURE.md has no `{heading}` section"))
        .split("\n## ")
        .next()
        .unwrap_or_default();
    section
        .lines()
        .filter_map(|line| line.strip_prefix("| `"))
        .filter_map(|rest| rest.split('`').next())
        .map(str::to_owned)
        .collect()
}

/// Every package of the workspace, and nothing else, is named in `ARCHITECTURE.md`'s
/// package table, and every crate `Cargo.lock` resolves from outside, and nothing else, in its
/// third-party list. Nothing enters Core without its reason written down, and the page never
/// names what is gone. This allow-list replaced, on 2026-11-22, fourteen deny-lists of crates
/// and modules removed long before (candle, wasmtime, reqwest, rmcp, the UI crates, ...).
#[test]
fn every_package_and_dependency_is_named_in_architecture_md() {
    let root = workspace_root();
    let mut members: HashSet<String> = workspace_graph().into_keys().collect();
    let lock = std::fs::read_to_string(root.join("Cargo.lock")).unwrap();
    let locked: HashSet<String> = lock
        .lines()
        .filter_map(|line| line.strip_prefix("name = \""))
        .map(|name| name.trim_end_matches('"').to_owned())
        .collect();
    let third_party: HashSet<String> = locked.difference(&members).cloned().collect();
    members.retain(|m| locked.contains(m));
    let sorted = |set: &HashSet<String>| {
        let mut v: Vec<String> = set.iter().cloned().collect();
        v.sort();
        v
    };
    assert_eq!(
        sorted(&names_in_section("## Packages")),
        sorted(&members),
        "ARCHITECTURE.md's package table names exactly the workspace's packages"
    );
    assert_eq!(
        sorted(&names_in_section("## Third-party crates")),
        sorted(&third_party),
        "ARCHITECTURE.md's third-party list names exactly the crates Cargo.lock resolves"
    );
}

fn non_test(text: &str) -> &str {
    text.find("#[cfg(test)]").map_or(text, |i| &text[..i])
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn rust_files(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Coverage floor ratchet: every non-facade crate must contain at least one
/// test (`#[test]`, `#[tokio::test]`, or `proptest!`). Pure re-export
/// facades (`susi-vendor-*` crates whose only job is `pub use upstream::*`)
/// are exempt — they carry no logic; their upstreams are tested upstream.
/// Implementation vendor crates (wasmer, candle, agents, models, web,
/// cloud, chrome, mcp-server, …) are NOT exempt: they hold real SUSI code.
#[test]
fn every_non_facade_crate_has_tests() {
    let root = workspace_root();
    let facades: HashSet<&str> = VENDOR_FACADES.iter().copied().collect();
    let mut bare = Vec::new();
    for dir in std::fs::read_dir(root.join("crates")).unwrap().flatten() {
        let crate_dir = dir.path();
        if !crate_dir.join("Cargo.toml").exists() {
            continue;
        }
        let name = crate_dir.file_name().unwrap().to_string_lossy().to_string();
        if facades.contains(name.as_str()) {
            continue;
        }
        let mut text = String::new();
        let mut files = Vec::new();
        rust_files(&crate_dir.join("src"), &mut files);
        rust_files(&crate_dir.join("tests"), &mut files);
        for f in files {
            text.push_str(&std::fs::read_to_string(&f).unwrap());
        }
        if !["#[test]", "#[tokio::test", "proptest!", "#[rstest]"]
            .iter()
            .any(|p| text.contains(p))
        {
            bare.push(name);
        }
    }
    bare.sort();
    assert!(
        bare.is_empty(),
        "every non-facade crate must carry tests; these have none: {bare:?}"
    );
}

// ── Cross-crate source mount ratchet ────────────────────────────────────
//
// A `#[path]` attribute that reaches into another crate's directory compiles
// a second private copy of that code, with its own types and statics. Shared
// code belongs in a crate reached through a Cargo edge. The count may only
// go down.

const CROSS_CRATE_SOURCE_MOUNTS_CEILING: usize = 0;

fn owning_crate(path: &std::path::Path) -> Option<PathBuf> {
    let crates = workspace_root().join("crates");
    let rel = path.strip_prefix(&crates).ok()?;
    rel.components().next().map(|c| crates.join(c.as_os_str()))
}

fn cross_crate_source_mounts() -> Vec<String> {
    let root = workspace_root();
    let mut files = Vec::new();
    rust_files(&root.join("crates"), &mut files);
    let mut mounts = Vec::new();
    for file in files {
        let text = std::fs::read_to_string(&file).unwrap();
        for line in text.lines() {
            let Some(rest) = line.split_once("#[path = \"").map(|(_, rest)| rest) else {
                continue;
            };
            let Some((mount, _)) = rest.split_once('"') else {
                continue;
            };
            let target = file.parent().unwrap().join(mount);
            let target = std::fs::canonicalize(&target).unwrap_or(target);
            let source = std::fs::canonicalize(&file).unwrap_or_else(|_| file.clone());
            if owning_crate(&source) != owning_crate(&target) {
                mounts.push(format!("{} -> {mount}", file.display()));
            }
        }
    }
    mounts.sort();
    mounts
}

#[test]
fn cross_crate_source_mounts_only_decrease() {
    let mounts = cross_crate_source_mounts();
    // Ceiling is 0: `len() <= 0` trips clippy::absurd_extreme_comparisons.
    assert!(
        mounts.is_empty(),
        "{} cross-crate #[path] mounts remain (ceiling {CROSS_CRATE_SOURCE_MOUNTS_CEILING}); \
         depend on the owning crate instead: {mounts:#?}",
        mounts.len()
    );
    eprintln!(
        "cross-crate source mounts: {} (ceiling {CROSS_CRATE_SOURCE_MOUNTS_CEILING})",
        mounts.len()
    );
}

// ── Allow audit trail ───────────────────────────────────────────────────
//
// AGENTS.md: "An #[allow] without a written justification is a compliance
// violation." Every allow of a denied panic-path lint must carry a `//`
// comment on its line, in the comment block right above it, or within the
// next few lines of the item it covers (the body's `Mandate 42: safe`
// note). Test-only files are exempt, as the lints are.

#[test]
fn every_panic_path_allow_is_justified() {
    const LINTS: &[&str] = &[
        "unwrap_used",
        "expect_used",
        "panic",
        "unreachable",
        "todo",
        "unimplemented",
        "wildcard_enum_match_arm",
    ];
    let root = workspace_root();
    let mut files = Vec::new();
    rust_files(&root.join("src"), &mut files);
    rust_files(&root.join("crates"), &mut files);
    let mut unjustified = Vec::new();
    for file in files {
        // Forward slashes on every host: a Windows path spelled with
        // backslashes made every crate's tests/ look like production code.
        let path = file.to_string_lossy().replace('\\', "/");
        if path.contains("/tests/") || path.contains("/benches/") || path.ends_with("_tests.rs") {
            continue;
        }
        let text = std::fs::read_to_string(&file).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if !(trimmed.starts_with("#[allow(") || trimmed.starts_with("#![allow(")) {
                continue;
            }
            if !LINTS
                .iter()
                .any(|lint| line.contains(&format!("clippy::{lint}")))
            {
                continue;
            }
            let same_line = line.contains("//");
            let above = i > 0 && lines[i - 1].trim_start().starts_with("//");
            let below = lines.iter().skip(i + 1).take(8).any(|l| l.contains("//"));
            if !(same_line || above || below) {
                unjustified.push(format!("{}:{}", path, i + 1));
            }
        }
    }
    assert!(
        unjustified.is_empty(),
        "panic-path #[allow]s without a written justification: {unjustified:?}"
    );
}

/// Mandate 49 regression guard: the worktree-workflow guard exists, is
/// executable, and every hook that must enforce it actually calls it.
#[cfg(unix)]
#[test]
fn worktree_workflow_guard_is_wired_into_every_hook() {
    use std::os::unix::fs::PermissionsExt;
    let root = workspace_root();
    let guard = root.join(".githooks/workflow-guard");
    let mode = std::fs::metadata(&guard)
        .expect("workflow-guard missing (Mandate 49)")
        .permissions()
        .mode();
    assert!(mode & 0o111 != 0, "workflow-guard must be executable");
    for (hook, stage) in [("pre-commit", "commit"), ("pre-merge-commit", "merge")] {
        let body = std::fs::read_to_string(root.join(".githooks").join(hook))
            .unwrap_or_else(|_| panic!("{hook} missing (Mandate 49)"));
        assert!(
            body.contains(&format!("workflow-guard\" {stage}")),
            "{hook} must call workflow-guard {stage} (Mandate 49)"
        );
    }
    // Main is unqualified development with no mandatory pre-push gate
    // (AGENTS.md, "Working here"), so pre-push no longer runs the
    // workflow guard's task-compliance and sync checks. It keeps the push
    // safety that guard gave: the native transport policy refuses branch
    // deletion, non-fast-forward branch updates and moved or deleted tags.
    let pre_push = std::fs::read_to_string(root.join(".githooks/pre-push"))
        .expect("pre-push missing (Mandate 49)");
    assert!(
        pre_push
            .lines()
            .any(|line| line.trim() == "exec push-policy \"$@\""),
        "pre-push must exec the native push-policy"
    );
    // The push-policy rules themselves moved with the tool to Susi Commercial
    // Apps (tools/core-xtask), which pins them; the hook still fails closed.
    assert!(
        pre_push.contains("refusing the push (transport safety)"),
        "pre-push must refuse a push when push-policy cannot be found or built"
    );
    // The guard is a stub for `susi workflow guard`: the rules are Rust's,
    // reached through `.githooks/run-susi`, which needs sh and git only.
    let guard_body = std::fs::read_to_string(&guard).unwrap_or_default();
    assert!(
        guard_body.contains("run-susi\" workflow guard"),
        "workflow-guard must exec `susi workflow guard` (Mandate 49)"
    );
    let runner = std::fs::metadata(root.join(".githooks/run-susi"))
        .expect("run-susi missing (Mandate 49)")
        .permissions()
        .mode();
    assert!(runner & 0o111 != 0, "run-susi must be executable");
    let setup = std::fs::read_to_string(root.join("scripts/setup-dev.sh")).unwrap_or_default();
    assert!(
        setup.contains("core.hooksPath .githooks"),
        "setup-dev.sh must enable the hooks (Mandate 49)"
    );
}

/// `-Clinker-features=-lld` is stable only on x86_64 Linux; on any other
/// target rustc rejects it, which failed v0.16.0's linux-aarch64 release build.
#[test]
fn linker_features_flag_is_x86_64_linux_only() {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".cargo/config.toml"),
    )
    .unwrap();
    let mut section = String::new();
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            section = line.to_string();
        } else if !line.starts_with('#') && line.contains("linker-features") {
            assert_eq!(
                section, "[target.x86_64-unknown-linux-gnu]",
                "linker-features is unstable outside x86_64 Linux: {line}"
            );
        }
    }
}

/// The brain-lanes markdown board was retired: lane ownership is one JSON file
/// per agent (no shared table to conflict on), its step logs live in the
/// local history, and nothing may point at the deleted file.
#[test]
fn brain_lanes_board_is_retired_into_data() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(
        !root.join("docs/brain-lanes.md").exists(),
        "docs/brain-lanes.md must stay deleted; use .agents/lanes/<AGENT>.json and `susi-commercial tasks`"
    );
    let mut agents = Vec::new();
    for entry in std::fs::read_dir(root.join(".agents/lanes")).unwrap() {
        let path = entry.unwrap().path();
        let lane: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let agent = lane["agent"].as_str().unwrap();
        assert_eq!(
            path.file_stem().unwrap().to_string_lossy(),
            agent,
            "lane file name must match its agent"
        );
        assert!(agent
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit()));
        assert!(
            !lane["stages"].as_array().unwrap().is_empty(),
            "{agent} lane needs at least one stage"
        );
        assert!(!lane["files"].as_array().unwrap().is_empty());
        agents.push(agent.to_string());
    }
    agents.sort();
    assert_eq!(agents, ["CLAUDE", "DEVIN"]);
    for rel in ["ARCHITECTURE.md", "AGENTS.md", "README.md"] {
        let text = std::fs::read_to_string(root.join(rel)).unwrap();
        assert!(
            !text.contains("brain-lanes.md"),
            "{rel} still points at the deleted brain-lanes.md"
        );
    }
}

/// Roadmap vectors say what and why; tasks are how it gets done. Every task
/// link must name a real vector, and every P0 vector that is not settled must
/// have an open task (so the most important work is always reachable from the queue).
#[test]
fn roadmap_vectors_and_tasks_stay_linked() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let roadmap: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join(".agents/roadmap.json")).unwrap())
            .unwrap();
    let vectors: Vec<(String, String)> = roadmap["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            (
                v["id"].as_str().unwrap().to_string(),
                v["priority"].as_str().unwrap_or("").to_string(),
            )
        })
        .collect();
    let mut linked = std::collections::HashSet::new();
    for dir in [".agents/tasks"] {
        let Ok(entries) = std::fs::read_dir(root.join(dir)) else {
            continue;
        };
        for e in entries.filter_map(Result::ok) {
            if e.path().extension().is_none_or(|x| x != "json") {
                continue;
            }
            let t: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(e.path()).unwrap()).unwrap();
            if let Some(v) = t["roadmap"].as_str() {
                assert!(
                    vectors.iter().any(|(id, _)| id == v),
                    "{} links roadmap vector {v}, which does not exist",
                    e.path().display()
                );
                linked.insert(v.to_string());
            }
        }
    }
    // A P0 vector is reachable from the queue (an open task links it) or settled on the
    // roadmap itself (its progress says DELIVERED or MOVED). Until 2026-11-22 a finished
    // record in .agents/tasks/done/ also counted; those records left, and every P0 vector they
    // covered was settled on the roadmap (54 delivered, 12 moved to Susi).
    let settled: std::collections::HashSet<&str> = roadmap["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| {
            v["progress"].as_str().is_some_and(|p| {
                let p = p.trim_start();
                p.starts_with("DELIVERED") || p.starts_with("MOVED")
            })
        })
        .filter_map(|v| v["id"].as_str())
        .collect();
    let uncovered: Vec<&str> = vectors
        .iter()
        .filter(|(id, p)| p == "P0" && !linked.contains(id) && !settled.contains(id.as_str()))
        .map(|(id, _)| id.as_str())
        .collect();
    assert!(
        uncovered.is_empty(),
        "P0 roadmap vectors neither settled nor linked by an open task: {uncovered:?}; \
         add a task whose record says \"roadmap\": \"<id>\""
    );
}

/// A stranger reads the README's first screen: before its first section it
/// says which repository this is, what Susi and Susi Tools are, what is public
/// and what is not, who builds the software and what the licence does not grant.
#[test]
fn readme_first_screen_says_what_this_repository_is() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = std::fs::read_to_string(root.join("README.md")).unwrap();
    let first_screen: String = readme
        .split("\n## ")
        .next()
        .unwrap()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for needle in [
        "This repository is Susi Core, the public Apache-2.0 derivative of Susi.",
        "| **Susi** | The product:",
        "No. Private and licensed.",
        "| **Susi Tools** | The development tooling:",
        "Yes. Apache-2.0",
        "Nothing in this repository is secret",
        "from Susi to Susi Core and Susi Tools, never the reverse",
        "The software is built by AI agents.",
        "no outside pull request, patch or commit is merged",
        "free to fork",
        "not to the Susi name or marks",
        "Not finished yet",
    ] {
        assert!(
            first_screen.contains(needle),
            "the README's first screen lost `{needle}`"
        );
    }
}

/// The README is the front door: it must cover the workflow, the task queue,
/// the ecosystem and brain commands, and point to the active contract without
/// growing a second history of policy changes.
#[test]
fn readme_covers_the_workflow_ecosystem_and_brain() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = std::fs::read_to_string(root.join("README.md")).unwrap();
    for needle in [
        // `local-workflow` and `xtask local-build debug` were required here until 2026-11-22,
        // when the rebuilt `susi-tools` program took over the loop and those commands left it.
        "susi-tools",
        "the Linux gates",
        "Task: T-<AGENT>-<n>",
        "actual combined tree",
        "Agents own authorized development",
        // `susi ecosystem scan`, `susi keys check`, `susi brain` and `SUSI_BUDGET` were
        // required here until 2026-11-22. Those commands hand their work to a runtime host
        // that left the private product that day, so the README stopped advertising them;
        // they return to this list when the new runtime serves them.
        "active engineering contract",
        "Operational history",
        "outside source",
    ] {
        assert!(readme.contains(needle), "README.md must cover `{needle}`");
    }
}

/// Core no longer ships a `vendor/` tree: model/TLS patches moved with their
/// implementations. Any remaining `[patch.crates-io]` entry must still be a
/// local path (never a Git host), and the lockfile must stay git-free.
#[test]
fn vendor_patches_are_local_and_preserve_upstream_licenses() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(
        !root.join("vendor").exists(),
        "core must not grow a vendor/ tree again after the socket slim"
    );
    let manifest = std::fs::read_to_string(root.join("Cargo.toml")).unwrap();
    // No patch table at all is the simplest local state (2026-11-22).
    let patch = manifest
        .split("[patch.crates-io]")
        .nth(1)
        .unwrap_or("")
        .split("\n[")
        .next()
        .unwrap();
    for line in patch
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        assert!(
            line.contains("path = \""),
            "nonlocal patch (path required): {line}"
        );
        assert!(!line.contains("git ="), "GitHub-dependent patch: {line}");
    }
    let lock = std::fs::read_to_string(root.join("Cargo.lock")).unwrap();
    assert!(
        !lock.contains("source = \"git+"),
        "development sources must resolve without a Git host"
    );
    assert!(
        !lock.contains("[[patch.unused]]"),
        "Cargo.lock must not carry unused [patch.crates-io] entries"
    );
}
/// Susi is licensed under Apache-2.0 and admits only Apache-2.0, MIT, and
/// BSD-family permissive licenses into its dependency graph. Strong copyleft
/// (GPL-2.0, GPL-3.0, AGPL), weak copyleft (LGPL, MPL-2.0), and license
/// exceptions (such as `auto_generate_cdp` under `GPL-3.0-or-later` or
/// `option-ext` under `MPL-2.0`) are strictly excluded, and every workspace
/// manifest explicitly declares `license = "Apache-2.0"`.
#[test]
fn workspace_enforces_apache_mit_bsd_license_policy() {
    let root = workspace_root();

    // 1. Every workspace crate manifest explicitly declares `license = "Apache-2.0"`.
    let mut manifests = vec![root.join("Cargo.toml")];
    for entry in std::fs::read_dir(root.join("crates")).expect("crates/") {
        let entry = entry.expect("entry");
        let p = entry.path().join("Cargo.toml");
        if p.is_file() {
            manifests.push(p);
        }
    }
    for path in manifests {
        let text = std::fs::read_to_string(&path).expect("read Cargo.toml");
        assert!(
            text.lines()
                .any(|line| line.trim() == "license = \"Apache-2.0\""),
            "{} must explicitly declare `license = \"Apache-2.0\"`",
            path.display()
        );
    }

    // 2. deny.toml admits no copyleft or non-Apache/MIT/BSD licenses, with one exception
    //    decided on 2026-11-22: Unicode-3.0 for unicode-ident alone, a crate only the
    //    procedural macros use while compiling. No other exception, and Unicode-3.0 never
    //    in the general allow list.
    let deny = std::fs::read_to_string(root.join("deny.toml")).expect("read deny.toml");
    assert!(
        !deny.contains("[[licenses.exceptions]]"),
        "deny.toml must not grant [[licenses.exceptions]]"
    );
    let exceptions: Vec<&str> = deny
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("{ allow = "))
        .collect();
    assert_eq!(
        exceptions,
        [r#"{ allow = ["Unicode-3.0"], crate = "unicode-ident" },"#],
        "deny.toml grants exactly one licence exception: Unicode-3.0 for unicode-ident"
    );
    let general = deny
        .split("allow = [")
        .nth(1)
        .and_then(|rest| rest.split(']').next())
        .expect("deny.toml has a general allow list");
    assert!(
        !general.contains("Unicode"),
        "Unicode-3.0 is allowed for unicode-ident alone, never in the general allow list"
    );
    for forbidden in [
        "GPL",
        "AGPL",
        "LGPL",
        "MPL-2.0",
        "BSL-1.0",
        "CC0-1.0",
        "Unlicense",
        "0BSD",
        "auto_generate_cdp",
    ] {
        assert!(
            !deny.contains(forbidden),
            "deny.toml must not admit {forbidden}"
        );
    }
    for required in [
        "\"Apache-2.0\"",
        "\"MIT\"",
        "\"BSD-2-Clause\"",
        "\"BSD-3-Clause\"",
    ] {
        assert!(
            deny.contains(required),
            "deny.toml must allow permissive license {required}"
        );
    }
    // Which crates the lock file may resolve at all is the allow-list in ARCHITECTURE.md
    // (`every_package_and_dependency_is_named_in_architecture_md`); their licences are checked
    // by `cargo deny`, a release gate.
}

/// Source may retain a release-compatible empty template, never operational facts.
#[test]
fn historical_evidence_is_not_shipped_or_generated_back_into_source() {
    let root = workspace_root();
    let read = |path: &str| std::fs::read_to_string(root.join(path)).unwrap();
    let template: serde_json::Value = serde_json::from_str(&read(".agents/evidence.json")).unwrap();
    assert_eq!(template["schema"], "susi/evidence/v1");
    assert_eq!(template["entries"], serde_json::json!([]));
    assert_eq!(
        template.as_object().unwrap().len(),
        3,
        "no historical summary metadata"
    );
    let schema: serde_json::Value =
        serde_json::from_str(&read(".agents/schemas/evidence.schema.json")).unwrap();
    assert_eq!(schema["properties"]["entries"]["minItems"], 0);
    // There is no build script since the program left (2026-11-22), so no history can
    // become a build input.
    assert!(!root.join("build.rs").exists(), "a build script came back");
    // Moved build/runtime readers are checked by commercial ownership_guards.
}

/// Core has no line limit: the size suite measures and reports, treats its other counts as
/// baselines a change may raise, saying why, and `cargo core-size` runs its report by name
/// (AGENTS.md, "The socket and its size").
#[test]
fn core_size_is_reported_never_refused() {
    let root = workspace_root();
    let flat = |path: &str| -> String {
        std::fs::read_to_string(root.join(path))
            .unwrap()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let size_suite = flat("tests/lean_core_ceilings.rs");
    for withdrawn in ["COMPLETE_SOCKET_CEILING", "COMPLETE_SOCKET_LIMIT"] {
        assert!(
            !size_suite.contains(withdrawn),
            "the withdrawn line limit returned to the size suite: {withdrawn}"
        );
    }
    for needle in [
        "fn complete_socket_size_is_reported()",
        "complete_socket_rust={complete_socket}",
        "This is a baseline, not a",
        "in the same commit and say why",
    ] {
        assert!(
            size_suite.contains(needle),
            "the size suite lost its report or its baseline rule: {needle}"
        );
    }
    assert!(
        flat(".cargo/config.toml")
            .contains("--test lean_core_ceilings complete_socket_size_is_reported -- --exact"),
        "`cargo core-size` must run the size report by its exact name"
    );
}

/// The workspace refuses an ignored test without a reason, an expected panic without its
/// message and a leftover `dbg!` (AGENTS.md, "Code, commits and tests"); the rules
/// themselves are pinned in `tests/agent_contract.rs`.
#[test]
fn workspace_lints_refuse_unreasoned_ignores_panics_and_dbg() {
    let manifest = std::fs::read_to_string(workspace_root().join("Cargo.toml")).unwrap();
    for lint in [
        "ignore_without_reason = \"deny\"",
        "should_panic_without_expect = \"deny\"",
        "dbg_macro = \"deny\"",
    ] {
        assert!(manifest.contains(lint), "Cargo.toml must deny {lint}");
    }
}

/// Mandate 63: every script that is not Rust is migration debt, and the
/// count only goes down. Lower the ceiling in the change that removes one.
#[test]
fn non_rust_scripts_only_decrease() {
    const SCRIPT_CEILING: usize = 67;
    fn walk(dir: &std::path::Path, found: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if path.is_dir() {
                if !matches!(name.as_ref(), "target" | ".git" | "node_modules") {
                    walk(&path, found);
                }
            } else if matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("sh" | "ps1" | "py")
            ) {
                found.push(path);
            }
        }
    }
    let mut found = Vec::new();
    walk(&workspace_root(), &mut found);
    assert!(
        found.len() <= SCRIPT_CEILING,
        "{} shell/PowerShell/Python files, ceiling {SCRIPT_CEILING}: a new script is not admitted (Mandate 63)",
        found.len()
    );
}

// ---------------------------------------------------------------------------
// Zero-UI boundary (CEO directive): core is a pure Rust CLI plus a standards
// protocol socket. It ships no GUI/TUI, no web assets, serves no HTML and
// carries no plugin content; every UI and plugin lives in
// Susi (repository `susi`). Plan: Susi's docs/zero-ui-migration-plan.md
// (items A1-A7).
// ---------------------------------------------------------------------------

/// Directories a zero-UI scan never descends into: build output, patched
/// upstream sources (`vendor/`), and tool/agent metadata (hidden dirs other
/// than the project's own hook and cargo configuration).
fn zero_ui_skips_dir(name: &str) -> bool {
    matches!(name, "target" | "out" | "vendor" | "node_modules")
        || (name.starts_with('.') && !matches!(name, ".githooks" | ".cargo"))
}

fn zero_ui_walk(dir: &std::path::Path, skip_vendor_facades: bool, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            if zero_ui_skips_dir(&name) || (skip_vendor_facades && name.starts_with("susi-vendor-"))
            {
                continue;
            }
            zero_ui_walk(&path, skip_vendor_facades, out);
        } else {
            out.push(path);
        }
    }
}

fn zero_ui_rel(path: &std::path::Path) -> String {
    path.strip_prefix(workspace_root())
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// BT-03: no web or front-end asset ships in core. `target/`, `out/`,
/// `vendor/`, tool/agent metadata (`.git/`, `.claude/`, `.agents/`, ...) and
/// the `susi-vendor-*` facade crates are not scanned. No test-fixture
/// exception is granted: no `tests/fixtures` UI asset exists, so none is
/// tolerated.
#[test]
fn core_ships_zero_ui_assets() {
    const UI_EXTENSIONS: &[&str] = &[
        "html", "htm", "css", "js", "mjs", "cjs", "jsx", "ts", "tsx", "vue", "svelte",
    ];
    let mut files = Vec::new();
    zero_ui_walk(&workspace_root(), true, &mut files);
    // A floor that proves the walk saw the tree, not a size rule: about 120 files are walked
    // since the program left (2026-11-22).
    assert!(files.len() > 60, "asset walk found too few files");
    let offenders: Vec<String> = files
        .iter()
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| UI_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
        })
        .map(|path| zero_ui_rel(path))
        .collect();
    assert!(
        offenders.is_empty(),
        "core ships zero UI assets; move these to Susi (repository susi):\n{}",
        offenders.join("\n")
    );
}

/// BT-04: no first-party, non-test Rust source serves HTML: no `text/html`
/// content type, no axum `Html` response and no HTML document literal.
/// Test modules (`#[cfg(test)]` onward), `tests/` trees and the
/// `susi-vendor-*` facades are not scanned.
#[test]
fn core_serves_no_html() {
    let root = workspace_root();
    let mut files = Vec::new();
    for dir in ["src", "crates"] {
        zero_ui_walk(&root.join(dir), true, &mut files);
    }
    let mut offenders = Vec::new();
    for path in files
        .iter()
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
    {
        let rel = zero_ui_rel(path);
        if rel.split('/').any(|part| part == "tests") || rel.ends_with("/tests.rs") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        let code = non_test(&text);
        let lower = code.to_ascii_lowercase();
        let html_type = code.match_indices("Html").any(|(i, _)| {
            let before_ok = i == 0 || !is_ident_byte(code.as_bytes()[i - 1]);
            let after = code.as_bytes().get(i + "Html".len()).copied();
            before_ok && matches!(after, Some(b'(' | b'<' | b';' | b','))
        });
        if lower.contains("text/html") || lower.contains("<!doctype html") || html_type {
            offenders.push(rel);
        }
    }
    assert!(
        offenders.is_empty(),
        "core serves no HTML (JSON/SSE/MCP/A2A only); offending sources:\n{}",
        offenders.join("\n")
    );
}

/// CEO directive 2026-11-16 (Mandate 72): core is Apache-2.0 and published one
/// way. The CEO alone holds push authority, nothing is accepted from outside,
/// and the world is free to fork. Every front door says so, and none of them
/// invites a pull request again.
#[test]
fn core_is_published_one_way_and_free_to_fork() {
    let root = workspace_root();
    // Wrapped prose: compare on single-spaced text.
    let flat = |path: &str| -> String {
        std::fs::read_to_string(root.join(path))
            .unwrap()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    for path in [
        "AGENTS.md",
        "CLAUDE.md",
        "GEMINI.md",
        "README.md",
        "VISION.md",
        "CONTRIBUTING.md",
        "CONTRIBUTORS.md",
        "docs/lean-core.md",
    ] {
        let text = flat(path);
        assert!(
            text.contains("published one way"),
            "{path}: must say core is published one way"
        );
        assert!(
            text.contains("free to fork") || text.contains("may fork"),
            "{path}: must say the world may fork core"
        );
    }
    for path in ["CONTRIBUTING.md", "AGENTS.md"] {
        let publication = flat(path);
        for requirement in [
            "designated publisher alone holds push authority",
            "Authorized agents may exercise that authority within their scope",
            "this policy grants no additional write access",
        ] {
            assert!(
                publication.contains(requirement),
                "{path}: preserve scoped publication authority: {requirement}"
            );
        }
    }
    for path in [
        "README.md",
        "CONTRIBUTING.md",
        "CONTRIBUTORS.md",
        "docs/lean-core.md",
    ] {
        let text = flat(path).to_lowercase();
        for invitation in [
            "welcomes contributions",
            "pull requests from a fork",
            "good first issue",
            "help wanted",
            "by contributing to",
            "contributing from outside",
            "never fork",
        ] {
            assert!(
                !text.contains(invitation),
                "{path}: `{invitation}` contradicts one-way publication"
            );
        }
    }
}
