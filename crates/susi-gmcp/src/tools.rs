//! Read-only file tools confined to one root directory: list a directory, read a file, search
//! file contents. Every path is resolved inside the root; a path that leaves it, by `..`, an
//! absolute name or a symbolic link, is refused.

use serde_json::{json, Value};
use std::path::{Component, Path, PathBuf};

/// Largest file `read_file` returns whole; larger files are cut at this size, and say so.
pub const READ_LIMIT: usize = 256 * 1024;
/// Files larger than this are skipped by `search_files`.
pub const SEARCH_FILE_LIMIT: u64 = 1024 * 1024;
/// Default and largest number of matches `search_files` returns.
pub const SEARCH_RESULTS: usize = 50;
/// Directory names `search_files` does not descend into.
const SKIPPED_DIRS: [&str; 4] = [".git", "target", "node_modules", ".susi"];

/// The tools, as MCP `tools/list` describes them.
pub fn catalog() -> Value {
    json!([
        {
            "name": "list_directory",
            "description": "List the entries of a directory inside the workspace root. Directories end with '/'.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Directory relative to the workspace root; default: the root." }
                }
            }
        },
        {
            "name": "read_file",
            "description": "Read a text file inside the workspace root (up to 256 KiB).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "File relative to the workspace root." }
                },
                "required": ["path"]
            }
        },
        {
            "name": "search_files",
            "description": "Find lines containing a text in the files under a directory of the workspace root.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "The text to find (case-insensitive)." },
                    "path": { "type": "string", "description": "Directory to search, relative to the workspace root; default: the root." },
                    "max_results": { "type": "integer", "description": "At most this many matching lines (1 to 50; default 50)." }
                },
                "required": ["query"]
            }
        }
    ])
}

/// A tool's outcome: its text, and whether it is an error the caller should see as one.
pub struct Outcome {
    /// What the tool returns, as text.
    pub text: String,
    /// True when the tool refused or failed.
    pub is_error: bool,
}

fn ok(text: String) -> Outcome {
    Outcome {
        text,
        is_error: false,
    }
}

fn refused(text: String) -> Outcome {
    Outcome {
        text,
        is_error: true,
    }
}

/// Run tool `name` with `args` inside `root`.
pub fn call(root: &Path, name: &str, args: &Value) -> Option<Outcome> {
    let path_arg = args.get("path").and_then(Value::as_str).unwrap_or("");
    let outcome = match name {
        "list_directory" => match resolve(root, path_arg) {
            Ok(dir) => list_directory(root, &dir),
            Err(why) => refused(why),
        },
        "read_file" => {
            if path_arg.is_empty() {
                refused("read_file needs a path".into())
            } else {
                match resolve(root, path_arg) {
                    Ok(file) => read_file(&file),
                    Err(why) => refused(why),
                }
            }
        }
        "search_files" => {
            let query = args.get("query").and_then(Value::as_str).unwrap_or("");
            let limit = args
                .get("max_results")
                .and_then(Value::as_u64)
                .map_or(SEARCH_RESULTS, |n| {
                    usize::try_from(n).unwrap_or(SEARCH_RESULTS)
                })
                .clamp(1, SEARCH_RESULTS);
            if query.is_empty() {
                refused("search_files needs a query".into())
            } else {
                match resolve(root, path_arg) {
                    Ok(dir) => search_files(root, &dir, query, limit),
                    Err(why) => refused(why),
                }
            }
        }
        _ => return None,
    };
    Some(outcome)
}

/// `relative` inside `root`, refused when it is absolute, climbs with `..`, or resolves (through
/// a symbolic link) to somewhere outside the root.
pub fn resolve(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let rel = Path::new(relative);
    for part in rel.components() {
        match part {
            Component::Normal(_) | Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(format!("{relative:?} is outside the workspace root"));
            }
        }
    }
    let root = root
        .canonicalize()
        .map_err(|e| format!("the workspace root is not readable: {e}"))?;
    let joined = root.join(rel);
    let real = joined
        .canonicalize()
        .map_err(|_| format!("{relative:?} does not exist in the workspace root"))?;
    if real.starts_with(&root) {
        Ok(real)
    } else {
        Err(format!("{relative:?} leads outside the workspace root"))
    }
}

fn display(root: &Path, path: &Path) -> String {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    path.strip_prefix(&root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn list_directory(root: &Path, dir: &Path) -> Outcome {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            return refused(format!(
                "{} is not a readable directory: {e}",
                display(root, dir)
            ))
        }
    };
    let mut names: Vec<String> = entries
        .flatten()
        .map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                format!("{name}/")
            } else {
                name
            }
        })
        .collect();
    names.sort();
    if names.is_empty() {
        ok("(empty directory)".into())
    } else {
        ok(names.join("\n"))
    }
}

fn read_file(file: &Path) -> Outcome {
    let bytes = match std::fs::read(file) {
        Ok(bytes) => bytes,
        Err(e) => return refused(format!("cannot read the file: {e}")),
    };
    if bytes.contains(&0) {
        return refused("the file is binary, not text".into());
    }
    let cut = bytes.len() > READ_LIMIT;
    let shown = bytes.get(..READ_LIMIT.min(bytes.len())).unwrap_or(&bytes);
    let mut text = String::from_utf8_lossy(shown).into_owned();
    if cut {
        text.push_str(&format!("\n[cut at {READ_LIMIT} of {} bytes]", bytes.len()));
    }
    ok(text)
}

fn search_files(root: &Path, dir: &Path, query: &str, limit: usize) -> Outcome {
    let needle = query.to_lowercase();
    let mut found = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    let mut files = Vec::new();
    while let Some(next) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&next) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            let name = entry.file_name().to_string_lossy().into_owned();
            if kind.is_dir() {
                if !SKIPPED_DIRS.contains(&name.as_str()) {
                    stack.push(entry.path());
                }
            } else if kind.is_file() {
                files.push(entry.path());
            }
        }
    }
    files.sort();
    'files: for file in files {
        if std::fs::metadata(&file).map_or(true, |m| m.len() > SEARCH_FILE_LIMIT) {
            continue;
        }
        let Ok(bytes) = std::fs::read(&file) else {
            continue;
        };
        if bytes.contains(&0) {
            continue;
        }
        let text = String::from_utf8_lossy(&bytes);
        for (n, line) in text.lines().enumerate() {
            if line.to_lowercase().contains(&needle) {
                let line = line.trim();
                let line: String = line.chars().take(200).collect();
                found.push(format!("{}:{}: {line}", display(root, &file), n + 1));
                if found.len() >= limit {
                    break 'files;
                }
            }
        }
    }
    if found.is_empty() {
        ok(format!("no line contains {query:?}"))
    } else {
        ok(found.join("\n"))
    }
}
