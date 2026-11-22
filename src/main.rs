#![forbid(unsafe_code)]
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::wildcard_enum_match_arm
    )
)]

//! `susi`, Susi Core's program: an MCP server over standard input and output, a local daemon
//! that serves it over HTTP, and its own installation. Every command here works on its own;
//! until 2026-11-22 the program also listed 55 commands that only handed their work to a
//! runtime host of the private product, and those left with that host.
#![allow(missing_docs)]

mod cli;
mod daemon;

use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "\
susi: Susi Core, the local-first AI OS socket (Apache-2.0)

Usage:
  susi mcp serve [--root <dir>]   an MCP server over standard input and output, with
                                  read-only file tools confined to <dir> (default: here)
  susi start [--root <dir>]       start the local daemon in the background: GET /health,
                                  POST /mcp (bearer token in $SUSI_HOME/api_token)
  susi status                     say whether the daemon is running
  susi stop                       stop the daemon
  susi install                    install this program into $SUSI_HOME/bin (~/.susi/bin)
  susi --version | --help

The daemon listens on 127.0.0.1, port 9090 + SUSI_PORT_OFFSET.
";

fn get_home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn usage_error(message: &str) -> ExitCode {
    eprintln!("susi: {message}\n\n{USAGE}");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    susi_paths::stdio::install_broken_pipe_guard();
    let words: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|w| w.to_string_lossy().into_owned())
        .collect();
    let words_ref: Vec<&str> = words.iter().map(String::as_str).collect();
    let options = |rest: &[String]| daemon::root_arg(rest);
    match words_ref.as_slice() {
        [] | ["--help" | "-h" | "help"] => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        ["--version" | "-V"] => {
            println!("susi {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        ["mcp", "serve", ..] => {
            options(&words[2..]).map_or_else(|e| usage_error(&e), daemon::mcp_serve)
        }
        ["daemon", "run", ..] => options(&words[2..]).map_or_else(|e| usage_error(&e), daemon::run),
        ["start", ..] => options(&words[1..]).map_or_else(|e| usage_error(&e), daemon::start),
        ["status"] => daemon::status(),
        ["stop"] => daemon::stop(),
        ["install"] => cli::install_cli::execute(),
        _ => usage_error(&format!("unknown command {:?}", words.join(" "))),
    }
}
