use crate::server::{Server, PROTOCOL_VERSIONS};
use serde_json::{json, Value};
use std::path::PathBuf;

struct Root(PathBuf);

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn root(tag: &str) -> Root {
    let dir = std::env::temp_dir().join(format!("susi-gmcp-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src/inner")).unwrap();
    std::fs::write(dir.join("README.md"), "Hello Susi\nsecond line\n").unwrap();
    std::fs::write(
        dir.join("src/lib.rs"),
        "fn main() {}\n// TODO: hello world\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/inner/data.bin"), [0u8, 1, 2, 3]).unwrap();
    std::fs::create_dir_all(dir.join(".git")).unwrap();
    std::fs::write(dir.join(".git/config"), "hello inside git\n").unwrap();
    Root(dir)
}

fn call(server: &Server, id: i64, method: &str, params: Value) -> Value {
    server
        .handle(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }))
        .unwrap()
}

fn tool_text(server: &Server, name: &str, args: Value) -> (String, bool) {
    let answer = call(
        server,
        9,
        "tools/call",
        json!({ "name": name, "arguments": args }),
    );
    let result = &answer["result"];
    (
        result["content"][0]["text"].as_str().unwrap().to_owned(),
        result["isError"].as_bool().unwrap(),
    )
}

#[test]
fn initialize_agrees_a_protocol_version_and_names_the_server() {
    let r = root("init");
    let server = Server::new(r.0.clone());
    let known = call(
        &server,
        1,
        "initialize",
        json!({ "protocolVersion": "2025-03-26" }),
    );
    assert_eq!(known["result"]["protocolVersion"], "2025-03-26");
    assert_eq!(known["result"]["serverInfo"]["name"], "susi-core");
    let unknown = call(
        &server,
        2,
        "initialize",
        json!({ "protocolVersion": "1999-01-01" }),
    );
    assert_eq!(unknown["result"]["protocolVersion"], PROTOCOL_VERSIONS[0]);
    assert!(server
        .handle(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }))
        .is_none());
    assert_eq!(call(&server, 3, "ping", json!({}))["result"], json!({}));
}

#[test]
fn the_three_tools_are_listed_with_their_schemas() {
    let r = root("list");
    let server = Server::new(r.0.clone());
    let answer = call(&server, 1, "tools/list", json!({}));
    let names: Vec<&str> = answer["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["list_directory", "read_file", "search_files"]);
    assert_eq!(
        answer["result"]["tools"][1]["inputSchema"]["required"],
        json!(["path"])
    );
}

#[test]
fn files_inside_the_root_are_listed_read_and_searched() {
    let r = root("read");
    let server = Server::new(r.0.clone());
    let (listing, err) = tool_text(&server, "list_directory", json!({}));
    assert!(
        !err && listing.contains("README.md") && listing.contains("src/"),
        "{listing}"
    );
    let (text, err) = tool_text(&server, "read_file", json!({ "path": "README.md" }));
    assert!(!err);
    assert_eq!(text, "Hello Susi\nsecond line\n");
    let (hits, err) = tool_text(&server, "search_files", json!({ "query": "HELLO" }));
    assert!(!err, "{hits}");
    assert!(hits.contains("README.md:1: Hello Susi"), "{hits}");
    assert!(
        hits.contains("src/lib.rs:2: // TODO: hello world"),
        "{hits}"
    );
    assert!(!hits.contains(".git"), "searched inside .git: {hits}");
    let (one, _) = tool_text(
        &server,
        "search_files",
        json!({ "query": "hello", "max_results": 1 }),
    );
    assert_eq!(one.lines().count(), 1, "{one}");
    let (binary, err) = tool_text(
        &server,
        "read_file",
        json!({ "path": "src/inner/data.bin" }),
    );
    assert!(err && binary.contains("binary"), "{binary}");
}

#[test]
fn a_path_that_leaves_the_root_is_refused() {
    let r = root("escape");
    let server = Server::new(r.0.join("src"));
    for path in [
        "../README.md",
        "/etc/passwd",
        "inner/../../README.md",
        "missing.txt",
    ] {
        let (text, err) = tool_text(&server, "read_file", json!({ "path": path }));
        assert!(err, "{path} was read: {text}");
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(r.0.join("README.md"), r.0.join("src/link.md")).unwrap();
        let (text, err) = tool_text(&server, "read_file", json!({ "path": "link.md" }));
        assert!(
            err && text.contains("outside"),
            "a link out of the root was followed: {text}"
        );
    }
}

#[test]
fn unknown_methods_tools_and_text_get_json_rpc_errors() {
    let r = root("errors");
    let server = Server::new(r.0.clone());
    assert_eq!(
        call(&server, 1, "resources/list", json!({}))["error"]["code"],
        -32601
    );
    assert_eq!(
        call(&server, 2, "tools/call", json!({ "name": "rm" }))["error"]["code"],
        -32602
    );
    assert_eq!(
        server.handle_text("{not json").unwrap()["error"]["code"],
        -32700
    );
    assert_eq!(
        server
            .handle(&json!({ "id": 3, "method": "ping" }))
            .unwrap()["error"]["code"],
        -32600
    );
}

#[test]
fn lines_in_give_one_answer_per_request_out() {
    let r = root("lines");
    let server = Server::new(r.0.clone());
    let input = concat!(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}"#,
        "\n",
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        "\n\n",
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
        "\n"
    );
    let mut out = Vec::new();
    server.serve_lines(input.as_bytes(), &mut out).unwrap();
    let lines: Vec<Value> = String::from_utf8(out)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0]["id"], 1);
    assert_eq!(lines[1]["id"], 2);
}
