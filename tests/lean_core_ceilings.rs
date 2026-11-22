#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Lean-core baselines: what the program `susi` compiles, and the size of the workspace in
//! packages, as last measured. Each constant here is a baseline, not a ceiling. A change may
//! raise one in the same commit, saying why in the constant's comment and in the commit body;
//! a change that lowers a count lowers its baseline too, so the record stays true. The history
//! of each number is in Git, not here (until 2026-11-22 these comments carried it, 170 lines).
//!
//! Core has no line limit (decision of 2026-11-22). The complete socket is measured and its size
//! reported, by every ordinary run and by `cargo core-size`; no size fails a test.
//!
//! The reach is measured from `cargo metadata` (normal and build edges, the way the program is
//! built; dev edges excluded), from the program's own package. Until 2026-11-22 it was measured
//! from the GAWD and GMCP packages; the GAWD ones left that day, reached by no program.
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The package whose program the reach is measured from.
const PROGRAM: &str = "susi";

/// Workspace packages the program does not compile, kept because Susi, the product, builds on
/// them (its manifest names them); `ARCHITECTURE.md` says so in its package table. Any other
/// package the program does not reach is bloat, and this suite fails on it.
const LIBRARIES_FOR_SUSI: [&str; 3] = ["susi-config", "susi-vendor-getrandom", "susi-vendor-libc"];

/// Third-party crates the program compiles. 2026-11-22: 21, measured when the packages no
/// program reached left (T-CLAUDE-437).
const THIRD_PARTY_BASELINE: usize = 21;
/// Workspace packages the program compiles, itself included. 2026-11-22: 7 (T-CLAUDE-437).
const WORKSPACE_BASELINE: usize = 7;
/// Workspace members in `Cargo.toml`. 2026-11-22: 19 -> 10 (T-CLAUDE-437: `susi-core`,
/// `susi-abi`, `susi-gawd-acp`, `susi-gawd-a2a` and five vendor facades left, reached by no
/// program and named by no Susi manifest).
const WORKSPACE_MEMBERS_BASELINE: usize = 10;
/// Packages resolved in `Cargo.lock`. 2026-11-22: 89 -> 33 (T-CLAUDE-437).
const CARGO_LOCK_PACKAGES_BASELINE: usize = 33;
/// `susi-vendor-*` facades the program's package names directly. 2026-11-22: 3 (T-CLAUDE-437).
const DIRECT_FACADE_EDGES_BASELINE: usize = 3;

/// The failure text for a count above its recorded baseline. It tells the
/// agent what to do, because a baseline is not a ceiling.
fn above_baseline(what: &str, now: usize, constant: &str, baseline: usize) -> String {
    format!(
        "{what}: {now}, above the recorded baseline {baseline}. This is a baseline, not a \
         ceiling: when the change needs it, raise `{constant}` in tests/lean_core_ceilings.rs \
         to {now} in the same commit and say why, in the constant's comment and in the commit \
         body. When the change does not need it, remove what raised the count"
    )
}

/// Advice, never a failure: a count that fell below its baseline.
fn note_below_baseline(what: &str, now: usize, constant: &str, baseline: usize) {
    if now < baseline {
        eprintln!(
            "lean-core: {what} fell to {now}, below the recorded baseline {baseline}; lower \
             `{constant}` in tests/lean_core_ceilings.rs to {now} in this change so the record \
             stays true"
        );
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn metadata() -> Value {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Command::new(cargo)
        .current_dir(workspace_root())
        .args(["metadata", "--format-version", "1", "--offline", "--locked"])
        .output()
        .expect("cargo metadata runs");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("cargo metadata is JSON")
}

struct Reach {
    members: BTreeSet<String>,
    workspace: BTreeSet<String>,
    third_party: BTreeSet<String>,
    direct_facades: BTreeSet<String>,
}

/// The normal and build closure of the program's package.
fn program_reach(metadata: &Value) -> Reach {
    let member_ids: BTreeSet<&str> = metadata["workspace_members"]
        .as_array()
        .expect("members")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let names: BTreeMap<&str, &str> = metadata["packages"]
        .as_array()
        .expect("packages")
        .iter()
        .map(|p| {
            (
                p["id"].as_str().expect("id"),
                p["name"].as_str().expect("name"),
            )
        })
        .collect();
    let nodes: BTreeMap<&str, &Value> = metadata["resolve"]["nodes"]
        .as_array()
        .expect("resolve nodes")
        .iter()
        .map(|n| (n["id"].as_str().expect("node id"), n))
        .collect();
    let built_edge = |dep: &Value| {
        dep["dep_kinds"].as_array().is_none_or(|kinds| {
            kinds
                .iter()
                .any(|k| matches!(k["kind"].as_str(), None | Some("normal" | "build")))
        })
    };
    let root = member_ids
        .iter()
        .copied()
        .find(|id| names[id] == PROGRAM)
        .expect("the program's package is a workspace member");
    let mut direct_facades = BTreeSet::new();
    for dep in nodes[root]["deps"].as_array().expect("deps") {
        let id = dep["pkg"].as_str().expect("pkg");
        if built_edge(dep) && names[id].starts_with("susi-vendor-") {
            direct_facades.insert(names[id].to_owned());
        }
    }
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut stack = vec![root];
    while let Some(id) = stack.pop() {
        if !seen.insert(id) {
            continue;
        }
        for dep in nodes[id]["deps"].as_array().expect("deps") {
            if built_edge(dep) {
                stack.push(dep["pkg"].as_str().expect("pkg"));
            }
        }
    }
    let (workspace, third_party): (Vec<&str>, Vec<&str>) =
        seen.iter().copied().partition(|id| member_ids.contains(id));
    let name_all = |ids: Vec<&str>| -> BTreeSet<String> {
        ids.into_iter().map(|id| names[id].to_owned()).collect()
    };
    Reach {
        members: name_all(member_ids.iter().copied().collect()),
        workspace: name_all(workspace),
        third_party: name_all(third_party),
        direct_facades,
    }
}

/// Every workspace package is compiled by the program or is one Susi builds on: nothing in the
/// workspace is there for nobody.
#[test]
fn every_package_is_used_by_the_program_or_by_susi() {
    let reach = program_reach(&metadata());
    let unused: Vec<&String> = reach
        .members
        .iter()
        .filter(|m| !reach.workspace.contains(*m) && !LIBRARIES_FOR_SUSI.contains(&m.as_str()))
        .collect();
    assert!(
        unused.is_empty(),
        "packages neither the program `{PROGRAM}` compiles nor Susi builds on: {unused:?}; \
         remove them, or name them in LIBRARIES_FOR_SUSI and in ARCHITECTURE.md with the reason"
    );
    for library in LIBRARIES_FOR_SUSI {
        assert!(
            reach.members.contains(library),
            "{library} is listed as a library for Susi but is not a workspace member"
        );
    }
}

/// The dependency reach and the workspace's size in packages, against the recorded baselines.
/// A count above its baseline fails with what to do: raise the baseline in the same commit and
/// say why, or remove what raised it.
#[test]
fn program_dependency_reach_stays_within_its_recorded_baselines() {
    let meta = metadata();
    let lock_packages = meta["packages"].as_array().expect("packages").len();
    let reach = program_reach(&meta);
    eprintln!(
        "lean-core: the program compiles {} workspace packages, {} third-party crates, {} direct facade edges (workspace members: {}, Cargo.lock packages: {})",
        reach.workspace.len(),
        reach.third_party.len(),
        reach.direct_facades.len(),
        reach.members.len(),
        lock_packages
    );
    for (what, now, constant, baseline, named) in [
        (
            "workspace members declared in Cargo.toml",
            reach.members.len(),
            "WORKSPACE_MEMBERS_BASELINE",
            WORKSPACE_MEMBERS_BASELINE,
            Some(&reach.members),
        ),
        (
            "packages resolved in Cargo.lock",
            lock_packages,
            "CARGO_LOCK_PACKAGES_BASELINE",
            CARGO_LOCK_PACKAGES_BASELINE,
            None,
        ),
        (
            "third-party crates the program compiles",
            reach.third_party.len(),
            "THIRD_PARTY_BASELINE",
            THIRD_PARTY_BASELINE,
            Some(&reach.third_party),
        ),
        (
            "workspace packages the program compiles",
            reach.workspace.len(),
            "WORKSPACE_BASELINE",
            WORKSPACE_BASELINE,
            Some(&reach.workspace),
        ),
        (
            "vendor facades the program names directly",
            reach.direct_facades.len(),
            "DIRECT_FACADE_EDGES_BASELINE",
            DIRECT_FACADE_EDGES_BASELINE,
            Some(&reach.direct_facades),
        ),
    ] {
        assert!(
            now <= baseline,
            "{}{}",
            above_baseline(what, now, constant, baseline),
            named.map_or_else(String::new, |names| format!(". Counted: {names:?}"))
        );
        note_below_baseline(what, now, constant, baseline);
    }
}

fn physical_lines(path: &Path) -> usize {
    let bytes = std::fs::read(path).expect("tracked file should be readable");
    bytes.iter().filter(|&&b| b == b'\n').count()
        + usize::from(!bytes.is_empty() && !bytes.ends_with(b"\n"))
}

fn is_test_rust(rel: &str) -> bool {
    let parts: Vec<&str> = rel.split('/').collect();
    let Some((&file, dirs)) = parts.split_last() else {
        return false;
    };
    dirs.contains(&"tests") || file == "tests.rs" || file.ends_with("_tests.rs")
}

/// Physical lines by repository category (`git ls-files`), printed for the record: runtime Rust
/// per package, vendor facades, tests and tooling. Includes untracked, non-ignored files so a new
/// unstaged source file is not invisible. No size fails this: core has no line limit.
#[test]
fn repository_size_is_reported() {
    let size = repository_size();
    assert!(
        size.complete_socket > 0,
        "the size report counted no runtime source"
    );
}

/// `cargo core-size`: measure the complete socket and report it. All first-party runtime and
/// facade code counts. It never fails on the number, whatever it is: core has no line limit. It
/// fails only when the measurement itself is broken (a package counted as empty).
#[test]
#[ignore = "the size report of cargo core-size; ordinary runs already print the same measurement"]
fn complete_socket_size_is_reported() {
    let size = repository_size();
    eprintln!(
        "core-size: the complete socket is {} physical runtime Rust lines (reported; core has no line limit)",
        size.complete_socket
    );
}

/// What one measurement of the repository returns.
struct RepositorySize {
    /// First-party runtime Rust plus the in-repository vendor facades.
    complete_socket: usize,
}

fn repository_size() -> RepositorySize {
    let root = workspace_root();
    let output = Command::new("git")
        .current_dir(&root)
        .args([
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
        ])
        .output()
        .expect("git ls-files runs");
    assert!(
        output.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut runtime_rust = 0usize;
    let mut facade_rust = 0usize;
    let mut test_rust = 0usize;
    let mut tooling_rust = 0usize;
    let mut other_lines = 0usize;
    let mut runtime_by_package: BTreeMap<String, usize> = BTreeMap::new();

    for raw in output.stdout.split(|&b| b == 0).collect::<BTreeSet<_>>() {
        if raw.is_empty() {
            continue;
        }
        let rel = String::from_utf8_lossy(raw);
        let path = root.join(rel.as_ref());
        if !path.is_file() {
            continue;
        }
        let lines = physical_lines(&path);
        if !rel.ends_with(".rs") {
            other_lines += lines;
        } else if rel.starts_with("toolchain/") || rel == "build.rs" {
            tooling_rust += lines;
        } else if is_test_rust(&rel) {
            test_rust += lines;
        } else {
            let package = if rel.starts_with("crates/") {
                rel.split('/').nth(1).expect("crate path")
            } else {
                "susi (root)"
            };
            if package.starts_with("susi-vendor-") {
                facade_rust += lines;
            } else {
                runtime_rust += lines;
            }
            *runtime_by_package.entry(package.to_owned()).or_default() += lines;
        }
    }

    eprintln!(
        "lean-core categories: runtime_rust={runtime_rust}, vendor_facade_rust={facade_rust}, test_rust={test_rust}, tooling_rust={tooling_rust}, other_files={other_lines}"
    );
    // A missing or empty inventory must not pass as a small measurement.
    for package in [
        "susi (root)",
        "susi-gmcp",
        "susi-paths",
        "susi-error",
        "susi-config",
    ] {
        assert!(
            runtime_by_package
                .get(package)
                .is_some_and(|&lines| lines > 0),
            "runtime inventory is incomplete: no source counted for {package}"
        );
    }
    let complete_socket = runtime_rust + facade_rust;
    eprintln!("lean-core size: complete_socket_rust={complete_socket} (reported; no line limit)");
    let mut packages: Vec<_> = runtime_by_package.into_iter().collect();
    packages.sort_by(|(a_name, a_size), (b_name, b_size)| {
        b_size.cmp(a_size).then_with(|| a_name.cmp(b_name))
    });
    for (package, lines) in packages {
        eprintln!("lean-core runtime: {package} = {lines} lines");
    }
    RepositorySize { complete_socket }
}
