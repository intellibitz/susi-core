// Tests use assertions and unwraps to report invalid repository fixtures directly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Every task record in `.agents/tasks/` has the shape the task queue reads: required `id`,
//! `title`, `accept.cmd`, `created_by` and `created_unix`; optional fields as the queue defaults
//! them. The queue is Susi Tools' (`susi-tools tasks`); a malformed record here is refused
//! before it reaches it.

use serde::Deserialize;
use std::path::Path;

/// A task record as the queue deserializes it (unknown fields are allowed).
#[derive(Deserialize)]
#[allow(dead_code)] // the fields are the schema; deserializing them is the check
struct Task {
    id: String,
    title: String,
    #[serde(default)]
    goal: String,
    #[serde(default)]
    size: Option<String>,
    #[serde(default)]
    deps: Vec<String>,
    accept: Accept,
    #[serde(default)]
    roadmap: Option<String>,
    created_by: String,
    created_unix: u64,
    #[serde(default)]
    authored: bool,
    #[serde(default)]
    closed: Option<Closed>,
}

#[derive(Deserialize)]
struct Accept {
    cmd: Vec<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)] // the fields are the schema; deserializing them is the check
struct Closed {
    at_unix: u64,
    commit: String,
    by: String,
}

/// The queue's acceptance rule as it applies here: a `cargo` argv. Susi Tools also admits a
/// script under `scripts/`; Core has none (`scripts/` left on 2026-11-23).
fn accept_allowed(cmd: &[String]) -> bool {
    cmd.first().is_some_and(|program| program == "cargo") && cmd.iter().all(|a| !a.contains('\0'))
}

fn validation_error(bytes: &[u8]) -> Option<String> {
    match serde_json::from_slice::<Task>(bytes) {
        Err(error) => Some(error.to_string()),
        Ok(task) if !accept_allowed(&task.accept.cmd) => {
            Some("accept.cmd must be an allowed argv, not a list of shell commands".into())
        }
        Ok(_) => None,
    }
}

#[test]
fn every_task_record_has_the_shape_the_queue_reads() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    // The task queue reads this directory alone: closing a task deletes its record.
    let mut invalid = Vec::new();
    for entry in std::fs::read_dir(root.join(".agents/tasks")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|extension| extension != "json") {
            continue;
        }
        if let Some(error) = validation_error(&std::fs::read(&path).unwrap()) {
            invalid.push(format!("{}: {error}", path.display()));
        }
    }
    invalid.sort();
    assert!(
        invalid.is_empty(),
        "invalid task records; repair their provenance and argv:\n{}",
        invalid.join("\n")
    );
}

fn valid_record() -> serde_json::Value {
    serde_json::json!({
        "id": "T-TEST-1",
        "title": "test record",
        "accept": {"cmd": ["cargo", "test", "--locked"]},
        "created_by": "TEST",
        "created_unix": 1
    })
}

#[test]
fn validation_requires_creation_fields_and_real_argv() {
    assert!(validation_error(&serde_json::to_vec(&valid_record()).unwrap()).is_none());
    for field in ["created_by", "created_unix"] {
        let mut record = valid_record();
        record.as_object_mut().unwrap().remove(field);
        let error = validation_error(&serde_json::to_vec(&record).unwrap()).unwrap();
        assert!(error.contains(field), "{error}");
    }
    let mut record = valid_record();
    record["accept"]["cmd"] = serde_json::json!(["cargo test --locked", "cargo clippy"]);
    assert!(validation_error(&serde_json::to_vec(&record).unwrap())
        .unwrap()
        .contains("allowed argv"));
}
