//! The commit-message hook is the loop's first enforcement layer (every
//! commit names a `Task: T-<AGENT>-<n>` trailer of an open task), and only its
//! wiring used to be asserted. These run the real hook in real temporary
//! repositories. The primary-checkout refusal and the push rules are the
//! workflow guard (`susi-commercial workflow guard`, Susi Commercial Apps'
//! dev workflow since the lean-core move), tested there.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

mod common;

fn git(dir: &Path, args: &[&str]) {
    let out = command(dir, "git", args, "", &[]);
    assert_eq!(out.0, 0, "git {args:?}: {}", out.2);
}

/// Runs `program` with `stdin` piped in; returns (code, stdout, stderr).
fn command(
    dir: &Path,
    program: &str,
    args: &[&str],
    stdin: &str,
    envs: &[(&str, &str)],
) -> (i32, String, String) {
    // A hook runs as git runs it: under Git for Windows bash on Windows.
    let mut cmd = if program.contains(".githooks") {
        common::script_command(Path::new(program))
    } else {
        std::process::Command::new(program)
    };
    cmd.args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@t")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@t")
        .env_remove("SUSI_HOME");
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let out = common::run_script_bounded(&mut cmd, stdin.as_bytes(), common::SCRIPT_BOUND);
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// The real hook from this repository, run inside a temporary repo.
fn hook(
    repo: &Path,
    name: &str,
    args: &[&str],
    stdin: &str,
    envs: &[(&str, &str)],
) -> (i32, String, String) {
    command(
        repo,
        repo.join(".githooks").join(name).to_str().unwrap(),
        args,
        stdin,
        envs,
    )
}

struct Fixture {
    root: PathBuf,
    work: PathBuf,
}

impl Fixture {
    /// A repository with this repo's real commit-message hook installed.
    fn new(tag: &str) -> Self {
        let root = std::env::temp_dir().join(format!("susi-hooks-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let work = root.join("work");
        std::fs::create_dir_all(&work).unwrap();
        git(&work, &["init", "--quiet", "-b", "main"]);
        git(&work, &["commit", "--allow-empty", "--quiet", "-m", "init"]);

        // The real hooks, and every script they call.
        let src = Path::new(env!("CARGO_MANIFEST_DIR"));
        std::fs::create_dir_all(work.join(".githooks")).unwrap();
        for name in [
            "commit-msg",
            "mandate-checks",
            "pre-push",
            "core-boundary-guard",
        ] {
            std::fs::copy(
                src.join(".githooks").join(name),
                work.join(".githooks").join(name),
            )
            .unwrap();
        }
        std::fs::write(work.join(".susi-repo-role"), "public-core\n").unwrap();
        std::fs::create_dir_all(work.join("scripts")).unwrap();
        for name in ["check-task-scope.sh", "check-worker-identity.sh"] {
            std::fs::copy(
                src.join("scripts").join(name),
                work.join("scripts").join(name),
            )
            .unwrap();
        }
        // After the setup commit, so the hook does not judge it.
        git(&work, &["config", "core.hooksPath", ".githooks"]);
        Self { root, work }
    }

    fn message(&self, body: &str) -> String {
        let path = self.root.join("MSG");
        std::fs::write(&path, body).unwrap();
        path.to_str().unwrap().to_string()
    }

    fn open_task(&self, id: &str) {
        std::fs::create_dir_all(self.work.join(".agents/tasks")).unwrap();
        std::fs::write(
            self.work.join(format!(".agents/tasks/{id}.json")),
            serde_json::json!({
                "id": id, "title": "x", "goal": "x", "size": "s",
                "accept": {"cmd": ["cargo", "--version"]},
                "created_by": "TEST", "created_unix": 0
            })
            .to_string(),
        )
        .unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn a_commit_message_without_a_task_trailer_is_refused() {
    let f = Fixture::new("msg");
    let msg = f.message("feat: do something\n");
    let (code, _, err) = hook(&f.work, "commit-msg", &[&msg], "", &[]);
    assert_ne!(code, 0, "a trailer-less commit must be refused: {err}");
    assert!(err.contains("no task"), "{err}");
    assert!(err.contains("Task: T-<AGENT>-<n>"), "{err}");
}

#[test]
fn a_trailer_naming_an_open_task_is_accepted() {
    let f = Fixture::new("trailer");
    f.open_task("T-TEST-1");
    let msg = f.message("feat: do something\n\nTask: T-TEST-1\n");
    let (code, _, err) = hook(&f.work, "commit-msg", &[&msg], "", &[]);
    assert_eq!(code, 0, "{err}");

    // A closed task cannot be cited again: closing deletes its record.
    std::fs::remove_file(f.work.join(".agents/tasks/T-TEST-1.json")).unwrap();
    let (code, _, err) = hook(&f.work, "commit-msg", &[&msg], "", &[]);
    assert_ne!(code, 0);
    assert!(err.contains("not an open task"), "{err}");
}

/// What a pre-push case is handed: the commit before the fixture's commit, the fixture's
/// commit, and the all-zero id Git uses for "no object".
struct Push<'a> {
    base: &'a str,
    head: &'a str,
    zero: &'a str,
}

/// Run the real pre-push hook, with no `push-policy` anywhere on `PATH`, on one ref update
/// `line` builds, in a fixture of its own. Each case is its own test so they run side by
/// side: until 2026-11-22 the eight ran one after another in one test, 4 to 7 seconds of the
/// gate's 10.
fn pre_push(tag: &str, strict: bool, line: impl Fn(&Push) -> String) -> (i32, String) {
    let f = Fixture::new(tag);
    let base = command(&f.work, "git", &["rev-parse", "HEAD"], "", &[])
        .1
        .trim()
        .to_string();
    git(
        &f.work,
        &[
            "commit",
            "--allow-empty",
            "--quiet",
            "-m",
            "feat: standalone prepush fixture",
            "-m",
            "Task: LOCAL-prepush-fixture",
        ],
    );
    let head = command(&f.work, "git", &["rev-parse", "HEAD"], "", &[])
        .1
        .trim()
        .to_string();
    let zero = "0000000000000000000000000000000000000000";
    // Leave out every directory holding push-policy, so the hook's own checks run.
    let clean_path = std::env::var_os("PATH")
        .map(|raw| {
            std::env::join_paths(std::env::split_paths(&raw).filter(|dir| {
                !dir.join("push-policy").exists() && !dir.join("push-policy.exe").exists()
            }))
            .unwrap()
            .to_string_lossy()
            .into_owned()
        })
        .unwrap_or_default();
    let mut envs = vec![("PATH", clean_path.as_str())];
    if strict {
        envs.push(("SUSI_REQUIRE_PUSH_POLICY", "1"));
    }
    let stdin = line(&Push {
        base: &base,
        head: &head,
        zero,
    });
    let (code, _, err) = hook(
        &f.work,
        "pre-push",
        &["origin", "file:///tmp/bare"],
        &stdin,
        &envs,
    );
    (code, err)
}

fn refused_for_transport_safety((code, err): (i32, String)) {
    assert_ne!(code, 0, "the push must be refused: {err}");
    assert!(
        err.contains("refusing the push (transport safety)"),
        "{err}"
    );
}

#[test]
fn pre_push_admits_a_fast_forward() {
    let (code, err) = pre_push("prepush-ff", false, |p| {
        format!("refs/heads/main {} refs/heads/main {}\n", p.head, p.base)
    });
    assert_eq!(code, 0, "a fast-forward push must succeed: {err}");
}

#[test]
fn pre_push_refuses_a_rewind() {
    refused_for_transport_safety(pre_push("prepush-rewind", false, |p| {
        format!("refs/heads/main {} refs/heads/main {}\n", p.base, p.head)
    }));
}

#[test]
fn pre_push_refuses_deleting_a_branch() {
    refused_for_transport_safety(pre_push("prepush-delete-branch", false, |p| {
        format!("(delete) {} refs/heads/main {}\n", p.zero, p.head)
    }));
}

#[test]
fn pre_push_admits_a_new_tag() {
    let (code, err) = pre_push("prepush-new-tag", false, |p| {
        format!("refs/tags/v0.1.0 {} refs/tags/v0.1.0 {}\n", p.head, p.zero)
    });
    assert_eq!(code, 0, "a new tag push must succeed: {err}");
}

#[test]
fn pre_push_refuses_moving_a_tag() {
    refused_for_transport_safety(pre_push("prepush-move-tag", false, |p| {
        format!("refs/tags/v0.1.0 {} refs/tags/v0.1.0 {}\n", p.head, p.base)
    }));
}

#[test]
fn pre_push_refuses_deleting_a_tag() {
    refused_for_transport_safety(pre_push("prepush-delete-tag", false, |p| {
        format!("(delete) {} refs/tags/v0.1.0 {}\n", p.zero, p.head)
    }));
}

#[test]
fn pre_push_refuses_a_private_board_ref() {
    let board = ["refs", "board", "item-1"].join("/");
    let (code, err) = pre_push("prepush-board", false, |p| {
        format!("{board} {} {board} {}\n", p.head, p.zero)
    });
    assert_ne!(code, 0, "a private board ref push must be refused");
    assert!(err.contains("core-boundary-guard"), "{err}");
}

#[test]
fn pre_push_fails_closed_without_push_policy_in_strict_mode() {
    refused_for_transport_safety(pre_push("prepush-strict", true, |p| {
        format!("refs/heads/main {} refs/heads/main {}\n", p.head, p.base)
    }));
}
