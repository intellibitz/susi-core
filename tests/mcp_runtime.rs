#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Susi Core's own runtime, end to end through the program: `susi mcp serve` answers MCP over
//! standard input and output, and `susi start`, `status` and `stop` run the local daemon with
//! its open `/health` and its token-protected `/mcp`. Private home, private port block.

mod common;

use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

struct Place {
    dir: PathBuf,
}

impl Drop for Place {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn place(tag: &str) -> Place {
    let dir = std::env::temp_dir().join(format!("susi-runtime-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("work/src")).unwrap();
    std::fs::create_dir_all(dir.join("home")).unwrap();
    std::fs::write(dir.join("work/notes.txt"), "the plan is small\n").unwrap();
    std::fs::write(
        dir.join("work/src/main.rs"),
        "fn main() { println!(\"plan\"); }\n",
    )
    .unwrap();
    Place { dir }
}

fn susi(place: &Path, offset: u16) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_susi"));
    command
        .env("SUSI_HOME", place.join("home/.susi"))
        .env("SUSI_PORT_OFFSET", offset.to_string());
    susi_paths::test_env::isolate_home(&mut command, &place.join("home"));
    command
}

/// Zero configuration: with an empty home, no Susi variable and no argument but the command, the
/// program serves the folder it is started in and writes nothing anywhere. Until 2026-11-22 this
/// was claimed by tests that wrote their own stand-in files and checked those.
#[test]
fn mcp_serve_needs_no_setting_and_writes_nothing() {
    let p = place("zero-config");
    let mut command = Command::new(env!("CARGO_BIN_EXE_susi"));
    susi_paths::test_env::isolate_home(&mut command, &p.dir.join("home"));
    for (name, _) in std::env::vars_os() {
        if name.to_string_lossy().starts_with("SUSI_") {
            command.env_remove(&name);
        }
    }
    let mut child = command
        .args(["mcp", "serve"])
        .current_dir(p.dir.join("work"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    writeln!(
        input,
        "{}",
        json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"read_file","arguments":{"path":"notes.txt"}}})
    )
    .unwrap();
    drop(input);
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let answer: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        answer["result"]["content"][0]["text"],
        "the plan is small\n"
    );
    let written: Vec<_> = std::fs::read_dir(p.dir.join("home")).unwrap().collect();
    assert!(written.is_empty(), "the home stays empty: {written:?}");
}

#[test]
fn mcp_serve_answers_a_client_session_over_stdio() {
    let p = place("stdio");
    let mut child = susi(&p.dir, 0)
        .args(["mcp", "serve", "--root"])
        .arg(p.dir.join("work"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let session = [
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}),
        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
        json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"read_file","arguments":{"path":"notes.txt"}}}),
        json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"search_files","arguments":{"query":"plan"}}}),
        json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"read_file","arguments":{"path":"../home/.susi/api_token"}}}),
    ];
    let mut input = child.stdin.take().unwrap();
    for message in &session {
        writeln!(input, "{message}").unwrap();
    }
    drop(input);
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let answers: Vec<Value> = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(
        answers.len(),
        5,
        "one answer per request, none for the notification"
    );
    assert_eq!(answers[0]["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(answers[1]["result"]["tools"].as_array().unwrap().len(), 3);
    assert_eq!(
        answers[2]["result"]["content"][0]["text"],
        "the plan is small\n"
    );
    let hits = answers[3]["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        hits.contains("notes.txt:1:") && hits.contains("src/main.rs:1:"),
        "{hits}"
    );
    assert_eq!(
        answers[4]["result"]["isError"], true,
        "a read outside the root was served"
    );
}

fn http(port: u16, request: &str) -> (u16, String) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream.write_all(request.as_bytes()).unwrap();
    let mut text = String::new();
    stream.read_to_string(&mut text).unwrap();
    let code = text.split_whitespace().nth(1).unwrap().parse().unwrap();
    (
        code,
        text.split_once("\r\n\r\n")
            .map_or("", |(_, b)| b)
            .to_owned(),
    )
}

fn post_mcp(port: u16, token: Option<&str>, body: &str) -> (u16, String) {
    let auth = token.map_or_else(String::new, |t| format!("Authorization: Bearer {t}\r\n"));
    http(
        port,
        &format!(
            "POST /mcp HTTP/1.1\r\nHost: 127.0.0.1\r\n{auth}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ),
    )
}

#[test]
fn the_daemon_starts_on_its_own_serves_health_and_mcp_and_stops() {
    let p = place("daemon");
    let offset = common::reserve_port_block((41_000..41_200).step_by(10), |offset| {
        std::net::TcpListener::bind(("127.0.0.1", 9090 + offset)).is_ok()
    })
    .expect("a free port block");
    let port = 9090 + offset;
    let start = susi(&p.dir, offset)
        .arg("start")
        .arg("--root")
        .arg(p.dir.join("work"))
        .output()
        .unwrap();
    assert!(
        start.status.success(),
        "{}",
        String::from_utf8_lossy(&start.stderr)
    );
    assert!(String::from_utf8_lossy(&start.stdout).contains(&format!("127.0.0.1:{port}")));

    let (code, body) = http(
        port,
        "GET /health HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n",
    );
    assert_eq!(code, 200, "{body}");
    assert!(body.contains("\"status\":\"ok\""), "{body}");

    let token_file = p.dir.join("home/.susi/api_token");
    let token = std::fs::read_to_string(&token_file).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&token_file).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "the token is readable by others");
    }
    let call = r#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"list_directory","arguments":{}}}"#;
    assert_eq!(
        post_mcp(port, None, call).0,
        401,
        "/mcp answered without a token"
    );
    assert_eq!(
        post_mcp(port, Some("wrong"), call).0,
        401,
        "/mcp answered a wrong token"
    );
    let (code, body) = post_mcp(port, Some(token.trim()), call);
    assert_eq!(code, 200, "{body}");
    let answer: Value = serde_json::from_str(&body).unwrap();
    let listing = answer["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        listing.contains("notes.txt") && listing.contains("src/"),
        "{listing}"
    );

    let status = susi(&p.dir, offset).arg("status").output().unwrap();
    assert!(status.status.success());
    let stop = susi(&p.dir, offset).arg("stop").output().unwrap();
    assert!(
        stop.status.success(),
        "{}",
        String::from_utf8_lossy(&stop.stderr)
    );
    let after = susi(&p.dir, offset).arg("status").output().unwrap();
    assert_eq!(
        after.status.code(),
        Some(3),
        "the daemon still answers after stop"
    );
    assert!(!p.dir.join("home/.susi/daemon.json").exists());
}
