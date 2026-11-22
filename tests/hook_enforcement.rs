//! The Git hooks are stubs: each `.githooks/<stage>` runs `susi-tools hook <stage>` from Susi
//! Tools, where the rules live and are tested (moved there 2026-11-23, with the shell scripts
//! this folder and `scripts/` held). These pin the wiring: the stages Git runs here, the options
//! this repository asks for (`--worktrees` for commits and merges, Mandate 49), and that a hook
//! refuses when no installed `susi-tools` answers, rather than letting a change through unchecked.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;

/// Every stage this repository hooks, and the line that hands it to Susi Tools.
const STAGES: [(&str, &str); 5] = [
    ("commit-msg", "exec susi-tools hook commit-msg \"$1\""),
    ("post-checkout", "exec susi-tools hook post-checkout \"$@\""),
    ("pre-commit", "exec susi-tools hook pre-commit --worktrees"),
    (
        "pre-merge-commit",
        "exec susi-tools hook pre-merge-commit --worktrees",
    ),
    ("pre-push", "exec susi-tools hook pre-push \"$@\""),
];

fn hooks_dir() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/.githooks"))
}

#[test]
fn every_hook_is_a_stub_that_runs_its_stage_in_susi_tools() {
    let mut names: Vec<String> = std::fs::read_dir(hooks_dir())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    let expected: Vec<&str> = STAGES.iter().map(|(name, _)| *name).collect();
    assert_eq!(
        names, expected,
        "the hooks folder holds the stubs and nothing else"
    );
    for (name, line) in STAGES {
        let body = std::fs::read_to_string(hooks_dir().join(name)).unwrap();
        assert!(body.starts_with("#!/bin/sh\n"), "{name} is a POSIX sh stub");
        assert_eq!(
            body.lines().last(),
            Some(line),
            "{name} hands its stage to Susi Tools"
        );
        assert!(
            body.contains("susi-tools hook --help >/dev/null 2>&1 ||"),
            "{name} says so when no installed susi-tools has the hooks"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(hooks_dir().join(name))
                .unwrap()
                .permissions()
                .mode();
            assert!(mode & 0o111 != 0, "{name} must be executable");
        }
    }
}

#[cfg(unix)]
#[test]
fn a_hook_refuses_when_no_susi_tools_answers_and_advice_never_does() {
    let empty = std::env::temp_dir().join(format!("susi-hooks-no-tools-{}", std::process::id()));
    std::fs::create_dir_all(&empty).unwrap();
    for (name, _) in STAGES {
        let out = std::process::Command::new("/bin/sh")
            .arg(hooks_dir().join(name))
            .env("PATH", &empty)
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("install the current susi-tools"),
            "{name}: {stderr}"
        );
        let expected = if name == "post-checkout" { 0 } else { 1 };
        assert_eq!(out.status.code(), Some(expected), "{name}: {stderr}");
    }
    let _ = std::fs::remove_dir_all(&empty);
}
