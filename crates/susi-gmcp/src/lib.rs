#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

//! GMCP, Susi Core's MCP server: a Model Context Protocol server with read-only file tools over
//! one workspace root ([`server`], [`tools`]), served by `susi mcp serve` (standard input and
//! output) and by `susi start` (HTTP). A host adds its own tools beside the built-in ones with
//! [`server::Server::with_tool`] and the [`server::Tool`] trait, without editing this crate.
//! Until 2026-11-22 that seam was a process-wide plane bus in the `susi-core` package, which no
//! program used; it left with that package.

pub mod server;
pub mod tools;

#[cfg(test)]
mod tests;
