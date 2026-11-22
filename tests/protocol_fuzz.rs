//! What the program reads from outside never panics it: every line the MCP server is handed,
//! garbage included, gets a JSON-RPC answer or, for a notification, none. Until 2026-11-22 this
//! file fuzzed the swarm-event and claim formats of the `susi-core` package, which no program
//! read; they left with it.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use serde_json::Value;
use susi_gmcp::server::Server;

fn server() -> Server {
    Server::new(std::env::temp_dir().join("susi-protocol-fuzz"))
}

#[test]
fn garbage_lines_get_a_json_rpc_error_never_a_panic() {
    let server = server();
    let deep = format!("{}{}", "[".repeat(10_000), "]".repeat(10_000));
    let long = format!(
        "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"{}\"}}",
        "x".repeat(70_000)
    );
    let cases: Vec<&str> = vec![
        "",
        "{",
        "null",
        "[]",
        "\u{0}{}",
        "\"plain\"",
        "{\"id\":1}",
        "{\"jsonrpc\":\"1.0\",\"id\":1,\"method\":\"ping\"}",
        "{\"jsonrpc\":\"2.0\",\"id\":{},\"method\":\"tools/call\",\"params\":null}",
        "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{\"name\":7}}",
        "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{\"name\":\"read_file\",\"arguments\":{\"path\":\"../../etc/passwd\"}}}",
        &deep,
        &long,
    ];
    for line in cases {
        if let Some(answer) = server.handle_text(line) {
            assert_eq!(answer["jsonrpc"], "2.0", "{line:.80}");
            let failed = answer.get("error").is_some()
                || answer["result"]["isError"].as_bool() == Some(true);
            assert!(failed, "garbage must not succeed: {line:.80} -> {answer}");
        }
    }
    // Odd but well-formed arguments get an answer too.
    let odd = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{\"name\":\"search_files\",\"arguments\":{\"query\":\"x\",\"max_results\":-5}}}";
    assert!(server.handle_text(odd).is_some());
}

#[test]
fn a_notification_gets_no_answer_and_a_request_gets_its_own_id() {
    let server = server();
    assert!(server
        .handle_text("{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}")
        .is_none());
    let answer: Value = server
        .handle_text("{\"jsonrpc\":\"2.0\",\"id\":\"a-7\",\"method\":\"ping\"}")
        .unwrap();
    assert_eq!(answer["id"], "a-7");
}
