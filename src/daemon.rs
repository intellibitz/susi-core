//! The local daemon: `susi start` runs it in the background, `susi status` asks it, `susi stop`
//! ends it. It listens on 127.0.0.1 only, port `9090 + SUSI_PORT_OFFSET`, and serves:
//! - `GET /health`: `{"status":"ok", ...}`, without a token;
//! - `POST /mcp`: the MCP server of [`susi_gmcp::server`], with `Authorization: Bearer <token>`;
//! - `POST /shutdown`: ends the daemon, with the token.
//!
//! The token is `api_token` in the Susi home (`SUSI_HOME`, else `~/.susi`), made on first start
//! from the system's random source and readable by its owner only. The running daemon's port,
//! process and root are in `daemon.json` beside it.

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::{Duration, Instant};

const BASE_PORT: u16 = 9090;
const MAX_HEADER_BYTES: usize = 16 * 1024;
const MAX_BODY_BYTES: usize = 1024 * 1024;

/// The Susi home: `SUSI_HOME`, else `~/.susi`.
pub fn home() -> PathBuf {
    std::env::var_os("SUSI_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(susi_paths::SusiDirs::substrate_home)
}

/// `9090 + SUSI_PORT_OFFSET` (an offset of 0 to 50000).
pub fn port() -> Result<u16, String> {
    let offset = match std::env::var("SUSI_PORT_OFFSET") {
        Ok(raw) if !raw.trim().is_empty() => raw
            .trim()
            .parse::<u16>()
            .ok()
            .filter(|o| *o <= 50_000)
            .ok_or_else(|| format!("SUSI_PORT_OFFSET must be 0 to 50000, not {raw:?}"))?,
        Ok(_) | Err(_) => 0,
    };
    Ok(BASE_PORT + offset)
}

fn token_path(home: &Path) -> PathBuf {
    home.join("api_token")
}

/// The daemon's token, made on first use: 32 random bytes as hex, readable by its owner only.
pub fn token(home: &Path) -> Result<String, String> {
    let path = token_path(home);
    if let Ok(text) = std::fs::read_to_string(&path) {
        let text = text.trim().to_owned();
        if text.len() >= 32 {
            return Ok(text);
        }
    }
    let mut bytes = [0u8; 32];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut bytes))
        .map_err(|e| format!("cannot read the system's random source: {e}"))?;
    let token: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    std::fs::create_dir_all(home).map_err(|e| format!("cannot create {}: {e}", home.display()))?;
    write_private(&path, token.as_bytes())?;
    Ok(token)
}

fn write_private(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
        .open(path)
        .and_then(|mut f| f.write_all(bytes))
        .map_err(|e| format!("cannot write {}: {e}", path.display()))
}

struct Reply {
    status: &'static str,
    body: String,
}

fn json_reply(status: &'static str, body: &Value) -> Reply {
    Reply {
        status,
        body: body.to_string(),
    }
}

/// One HTTP request as the daemon reads it.
struct Request {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: String,
}

fn read_request(stream: &TcpStream) -> Result<Request, String> {
    let mut reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);
    let mut first = String::new();
    reader.read_line(&mut first).map_err(|e| e.to_string())?;
    let mut parts = first.split_whitespace();
    let method = parts.next().unwrap_or("").to_owned();
    let path = parts.next().unwrap_or("").to_owned();
    let mut headers = Vec::new();
    let mut seen = first.len();
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).map_err(|e| e.to_string())?;
        seen += n;
        if n == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        if seen > MAX_HEADER_BYTES {
            return Err("headers too large".into());
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.push((name.trim().to_ascii_lowercase(), value.trim().to_owned()));
        }
    }
    let length = headers
        .iter()
        .find(|(name, _)| name == "content-length")
        .and_then(|(_, v)| v.parse::<usize>().ok())
        .unwrap_or(0);
    if length > MAX_BODY_BYTES {
        return Err("body too large".into());
    }
    let mut body = vec![0u8; length];
    reader.read_exact(&mut body).map_err(|e| e.to_string())?;
    Ok(Request {
        method,
        path,
        headers,
        body: String::from_utf8_lossy(&body).into_owned(),
    })
}

fn authorized(headers: &[(String, String)], token: &str) -> bool {
    headers
        .iter()
        .find(|(name, _)| name == "authorization")
        .and_then(|(_, v)| v.strip_prefix("Bearer "))
        .is_some_and(|given| constant_time_eq(given.trim().as_bytes(), token.as_bytes()))
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn respond(mut stream: &TcpStream, reply: &Reply) {
    let _ = write!(
        stream,
        "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        reply.status,
        reply.body.len(),
        reply.body
    );
    let _ = stream.flush();
}

/// Run the daemon in the foreground over `root` until `POST /shutdown`.
pub fn run(root: PathBuf) -> ExitCode {
    match serve(root) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("susi daemon: {e}");
            ExitCode::FAILURE
        }
    }
}

fn serve(root: PathBuf) -> Result<(), String> {
    let home = home();
    let port = port()?;
    let token = Arc::new(token(&home)?);
    let root = root
        .canonicalize()
        .map_err(|e| format!("the workspace root {} is not readable: {e}", root.display()))?;
    let listener = TcpListener::bind(("127.0.0.1", port))
        .map_err(|e| format!("cannot listen on 127.0.0.1:{port}: {e}"))?;
    let record = json!({
        "pid": std::process::id(),
        "port": port,
        "root": root.display().to_string(),
        "version": env!("CARGO_PKG_VERSION"),
    });
    write_private(&home.join("daemon.json"), record.to_string().as_bytes())?;
    let server = Arc::new(susi_gmcp::server::Server::new(root));
    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
        let Request {
            method,
            path,
            headers,
            body,
        } = match read_request(&stream) {
            Ok(request) => request,
            Err(why) => {
                respond(
                    &stream,
                    &json_reply("400 Bad Request", &json!({ "error": why })),
                );
                continue;
            }
        };
        match (method.as_str(), path.as_str()) {
            ("GET", "/health") => respond(
                &stream,
                &json_reply(
                    "200 OK",
                    &json!({ "status": "ok", "service": "susi-core", "version": env!("CARGO_PKG_VERSION"), "pid": std::process::id() }),
                ),
            ),
            ("POST", "/mcp") | ("POST", "/shutdown") if !authorized(&headers, &token) => respond(
                &stream,
                &json_reply(
                    "401 Unauthorized",
                    &json!({ "error": "a bearer token is required" }),
                ),
            ),
            ("POST", "/mcp") => {
                let server = Arc::clone(&server);
                std::thread::spawn(move || match server.handle_text(&body) {
                    Some(answer) => respond(&stream, &json_reply("200 OK", &answer)),
                    None => respond(
                        &stream,
                        &Reply {
                            status: "202 Accepted",
                            body: String::new(),
                        },
                    ),
                });
            }
            ("POST", "/shutdown") => {
                respond(
                    &stream,
                    &json_reply("200 OK", &json!({ "status": "stopping" })),
                );
                let _ = std::fs::remove_file(home.join("daemon.json"));
                return Ok(());
            }
            _ => respond(
                &stream,
                &json_reply("404 Not Found", &json!({ "error": "no such path" })),
            ),
        }
    }
    Ok(())
}

/// One HTTP exchange with the local daemon: the status code and the body.
fn exchange(
    port: u16,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: &str,
) -> Result<(u16, String), String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).map_err(|e| e.to_string())?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let auth = token.map_or_else(String::new, |t| format!("Authorization: Bearer {t}\r\n"));
    write!(
        stream,
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\n{auth}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .map_err(|e| e.to_string())?;
    let mut text = String::new();
    stream
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    let code = text
        .split_whitespace()
        .nth(1)
        .and_then(|c| c.parse::<u16>().ok())
        .ok_or("no HTTP status")?;
    let body = text
        .split_once("\r\n\r\n")
        .map_or("", |(_, b)| b)
        .to_owned();
    Ok((code, body))
}

fn healthy(port: u16) -> bool {
    exchange(port, "GET", "/health", None, "").is_ok_and(|(code, _)| code == 200)
}

/// Start the daemon in the background over `root`, and wait until it answers.
pub fn start(root: PathBuf) -> ExitCode {
    let port = match port() {
        Ok(port) => port,
        Err(e) => {
            eprintln!("susi start: {e}");
            return ExitCode::from(2);
        }
    };
    if healthy(port) {
        println!("susi is already running on 127.0.0.1:{port}");
        return ExitCode::SUCCESS;
    }
    let program = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("susi start: cannot find this program: {e}");
            return ExitCode::FAILURE;
        }
    };
    let spawned = std::process::Command::new(program)
        .args(["daemon", "run", "--root"])
        .arg(&root)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
    if let Err(e) = spawned {
        eprintln!("susi start: cannot start the daemon: {e}");
        return ExitCode::FAILURE;
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if healthy(port) {
            println!("susi is running on 127.0.0.1:{port}");
            println!("  GET  http://127.0.0.1:{port}/health");
            println!(
                "  POST http://127.0.0.1:{port}/mcp  (Authorization: Bearer <{}>)",
                token_path(&home()).display()
            );
            return ExitCode::SUCCESS;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    eprintln!("susi start: the daemon did not answer on 127.0.0.1:{port} within 5 seconds");
    ExitCode::FAILURE
}

/// Say whether the daemon is running.
pub fn status() -> ExitCode {
    let port = match port() {
        Ok(port) => port,
        Err(e) => {
            eprintln!("susi status: {e}");
            return ExitCode::from(2);
        }
    };
    match exchange(port, "GET", "/health", None, "") {
        Ok((200, body)) => {
            println!("running on 127.0.0.1:{port}: {body}");
            ExitCode::SUCCESS
        }
        Ok((_, _)) | Err(_) => {
            println!("not running on 127.0.0.1:{port}");
            ExitCode::from(3)
        }
    }
}

/// Stop the daemon, and wait until it is gone.
pub fn stop() -> ExitCode {
    let port = match port() {
        Ok(port) => port,
        Err(e) => {
            eprintln!("susi stop: {e}");
            return ExitCode::from(2);
        }
    };
    if !healthy(port) {
        println!("susi is not running on 127.0.0.1:{port}");
        return ExitCode::SUCCESS;
    }
    let token = match token(&home()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("susi stop: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = exchange(port, "POST", "/shutdown", Some(&token), "") {
        eprintln!("susi stop: {e}");
        return ExitCode::FAILURE;
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if !healthy(port) {
            println!("susi stopped");
            return ExitCode::SUCCESS;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    eprintln!("susi stop: the daemon is still answering on 127.0.0.1:{port}");
    ExitCode::FAILURE
}

/// `susi mcp serve [--root <dir>]`: the MCP server over standard input and output.
pub fn mcp_serve(root: PathBuf) -> ExitCode {
    let root = match root.canonicalize() {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "susi mcp serve: the workspace root {} is not readable: {e}",
                root.display()
            );
            return ExitCode::from(2);
        }
    };
    let server = susi_gmcp::server::Server::new(root);
    let stdin = std::io::stdin();
    match server.serve_lines(stdin.lock(), std::io::stdout().lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("susi mcp serve: {e}");
            ExitCode::FAILURE
        }
    }
}

/// `--root <dir>` among `args`, else the current directory.
pub fn root_arg(args: &[String]) -> Result<PathBuf, String> {
    let mut it = args.iter();
    let mut root = None;
    while let Some(arg) = it.next() {
        if arg == "--root" {
            root = Some(PathBuf::from(it.next().ok_or("--root needs a directory")?));
        } else {
            return Err(format!("unknown option {arg:?}"));
        }
    }
    Ok(root.unwrap_or_else(|| PathBuf::from(".")))
}
