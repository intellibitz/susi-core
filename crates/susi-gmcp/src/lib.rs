#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

//! GMCP, Susi Core's MCP pillar: a Model Context Protocol server with read-only file tools over
//! one workspace root ([`server`], [`tools`]), served by `susi mcp serve` (standard input and
//! output) and by `susi start` (HTTP). The generic tool-dispatch entry point over the plane bus
//! stays for hosts that register their own tools.

pub mod server;
pub mod tools;

pub use susi_core::plane_bus::tools as plane_tools;
/// The generic MCP wire model.
pub mod mcp {
    /// MCP wire types.
    pub mod model;
}
pub use mcp as rmcp;

/// The existing tool-dispatch entry point over the canonical plane bus.
pub struct GmcpHost;
impl GmcpHost {
    /// Run tool `name` through the plane bus with `arg` as JSON (or as a string).
    pub fn dispatch(name: &str, arg: &str, workspace: &std::path::Path) -> String {
        let value = serde_json::from_str(arg).unwrap_or(serde_json::json!(arg));
        plane_tools::execute_tool(name, &value, workspace)
            .unwrap_or_else(|error| format!("[Error] {error}"))
    }
}

#[cfg(test)]
mod tests;
