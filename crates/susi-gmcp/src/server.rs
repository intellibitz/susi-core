//! A Model Context Protocol server: JSON-RPC 2.0 requests in, responses out, the same over
//! standard input and output (`susi mcp serve`) and over HTTP (`susi start`, `POST /mcp`).

use crate::tools;
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::path::PathBuf;

/// MCP protocol versions this server speaks, newest first.
pub const PROTOCOL_VERSIONS: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];

/// One MCP server over one workspace root.
pub struct Server {
    root: PathBuf,
}

impl Server {
    /// A server whose tools see only `root`.
    pub fn new(root: PathBuf) -> Server {
        Server { root }
    }

    /// The answer to one JSON-RPC message, or `None` for a notification.
    pub fn handle(&self, message: &Value) -> Option<Value> {
        let id = message.get("id").cloned();
        let method = message.get("method").and_then(Value::as_str).unwrap_or("");
        let params = message.get("params").cloned().unwrap_or_else(|| json!({}));
        let id = id?;
        if message.get("jsonrpc").and_then(Value::as_str) != Some("2.0") || method.is_empty() {
            return Some(error(id, -32600, "not a JSON-RPC 2.0 request"));
        }
        Some(match method {
            "initialize" => {
                let asked = params
                    .get("protocolVersion")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let version = PROTOCOL_VERSIONS
                    .iter()
                    .find(|v| **v == asked)
                    .copied()
                    .unwrap_or(PROTOCOL_VERSIONS[0]);
                result(
                    id,
                    json!({
                        "protocolVersion": version,
                        "capabilities": { "tools": { "listChanged": false } },
                        "serverInfo": { "name": "susi-core", "version": env!("CARGO_PKG_VERSION") },
                        "instructions": "Read-only file tools over one workspace root."
                    }),
                )
            }
            "ping" => result(id, json!({})),
            "tools/list" => result(id, json!({ "tools": tools::catalog() })),
            "tools/call" => {
                let name = params.get("name").and_then(Value::as_str).unwrap_or("");
                let args = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                match tools::call(&self.root, name, &args) {
                    Some(outcome) => result(
                        id,
                        json!({
                            "content": [{ "type": "text", "text": outcome.text }],
                            "isError": outcome.is_error
                        }),
                    ),
                    None => error(id, -32602, &format!("no tool named {name:?}")),
                }
            }
            other => error(id, -32601, &format!("method {other:?} is not served")),
        })
    }

    /// The answer to one line of JSON text, or `None` for a notification.
    pub fn handle_text(&self, text: &str) -> Option<Value> {
        match serde_json::from_str::<Value>(text) {
            Ok(message) => self.handle(&message),
            Err(e) => Some(error(Value::Null, -32700, &format!("not JSON: {e}"))),
        }
    }

    /// Serve newline-delimited JSON-RPC from `input` to `output` until the input ends.
    pub fn serve_lines(&self, input: impl BufRead, mut output: impl Write) -> std::io::Result<()> {
        for line in input.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            if let Some(answer) = self.handle_text(&line) {
                writeln!(output, "{answer}")?;
                output.flush()?;
            }
        }
        Ok(())
    }
}

fn result(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn error(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}
