//! A Model Context Protocol server: JSON-RPC 2.0 requests in, responses out, the same over
//! standard input and output (`susi mcp serve`) and over HTTP (`susi start`, `POST /mcp`).

use crate::tools;
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

/// MCP protocol versions this server speaks, newest first.
pub const PROTOCOL_VERSIONS: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];

/// A tool a host adds to the server beside the built-in file tools, without editing this crate:
/// `tools/list` shows its entry and `tools/call` runs it by name.
pub trait Tool: Send + Sync {
    /// The tool's `tools/list` entry: an object with `name`, `description` and `inputSchema`.
    fn describe(&self) -> Value;
    /// Run the tool with `args`; `root` is the server's workspace root.
    fn call(&self, root: &Path, args: &Value) -> tools::Outcome;
}

/// One MCP server over one workspace root.
pub struct Server {
    root: PathBuf,
    added: Vec<(String, Box<dyn Tool>)>,
}

impl Server {
    /// A server whose tools see only `root`.
    pub fn new(root: PathBuf) -> Server {
        Server {
            root,
            added: Vec::new(),
        }
    }

    /// The server with `tool` added. Refused when its entry has no name, or a built-in or an
    /// earlier added tool already has that name.
    pub fn with_tool(mut self, tool: Box<dyn Tool>) -> Result<Server, String> {
        let name = tool
            .describe()
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        if name.is_empty() {
            return Err("an added tool's entry needs a name".into());
        }
        let taken = tools::catalog()
            .as_array()
            .is_some_and(|built_in| built_in.iter().any(|t| t["name"] == name.as_str()))
            || self.added.iter().any(|(n, _)| *n == name);
        if taken {
            return Err(format!("a tool named {name:?} is already served"));
        }
        self.added.push((name, tool));
        Ok(self)
    }

    fn catalog(&self) -> Value {
        let mut all = tools::catalog();
        if let Some(list) = all.as_array_mut() {
            list.extend(self.added.iter().map(|(_, tool)| tool.describe()));
        }
        all
    }

    fn call_tool(&self, name: &str, args: &Value) -> Option<tools::Outcome> {
        tools::call(&self.root, name, args).or_else(|| {
            self.added
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, tool)| tool.call(&self.root, args))
        })
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
            "tools/list" => result(id, json!({ "tools": self.catalog() })),
            "tools/call" => {
                let name = params.get("name").and_then(Value::as_str).unwrap_or("");
                let args = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                match self.call_tool(name, &args) {
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
