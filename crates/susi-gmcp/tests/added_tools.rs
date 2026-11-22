//! A host adds its own tools to Core's MCP server without editing it: `tools/list` shows them
//! beside the built-in file tools, `tools/call` runs them, and a name already served is refused.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use serde_json::{json, Value};
use std::path::Path;
use susi_gmcp::server::{Server, Tool};
use susi_gmcp::tools::Outcome;

struct Echo;

impl Tool for Echo {
    fn describe(&self) -> Value {
        json!({
            "name": "echo",
            "description": "Say the text back, with the workspace root's last name.",
            "inputSchema": { "type": "object", "properties": { "text": { "type": "string" } } }
        })
    }

    fn call(&self, root: &Path, args: &Value) -> Outcome {
        let text = args.get("text").and_then(Value::as_str).unwrap_or("");
        let leaf = root
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        Outcome {
            text: format!("{text} from {leaf}"),
            is_error: text.is_empty(),
        }
    }
}

struct Named(&'static str);

impl Tool for Named {
    fn describe(&self) -> Value {
        json!({ "name": self.0, "description": "", "inputSchema": { "type": "object" } })
    }

    fn call(&self, _root: &Path, _args: &Value) -> Outcome {
        Outcome {
            text: String::new(),
            is_error: false,
        }
    }
}

fn ask(server: &Server, method: &str, params: Value) -> Value {
    server
        .handle(&json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params }))
        .unwrap()
}

#[test]
fn an_added_tool_is_listed_and_called_beside_the_built_in_ones() {
    let root = std::env::temp_dir().join("susi-gmcp-added-tools");
    let server = Server::new(root).with_tool(Box::new(Echo)).unwrap();

    let listed = ask(&server, "tools/list", json!({}));
    let names: Vec<&str> = listed["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    assert_eq!(
        names,
        ["list_directory", "read_file", "search_files", "echo"]
    );

    let called = ask(
        &server,
        "tools/call",
        json!({ "name": "echo", "arguments": { "text": "hello" } }),
    );
    assert_eq!(
        called["result"]["content"][0]["text"],
        "hello from susi-gmcp-added-tools"
    );
    assert_eq!(called["result"]["isError"], false);

    let refused = ask(
        &server,
        "tools/call",
        json!({ "name": "echo", "arguments": {} }),
    );
    assert_eq!(refused["result"]["isError"], true);

    let unknown = ask(&server, "tools/call", json!({ "name": "nothing" }));
    assert_eq!(unknown["error"]["code"], -32602);
}

#[test]
fn a_tool_name_already_served_or_missing_is_refused() {
    let server = || Server::new(std::env::temp_dir());
    assert!(server().with_tool(Box::new(Named("read_file"))).is_err());
    assert!(server().with_tool(Box::new(Named(""))).is_err());
    let twice = server()
        .with_tool(Box::new(Named("mine")))
        .unwrap()
        .with_tool(Box::new(Named("mine")));
    assert!(twice.is_err());
}
