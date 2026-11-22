#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]
#![allow(missing_docs)] // integration test crate: no public API to document

//! `.mcp.json` is read by every Claude Code session that opens this
//! repository: a laptop's, a teammate's, and a cloud session's disposable
//! container. It once named one laptop's build —
//! `F:/src/github.com/intellibitz/susi/target/debug/susi.exe`, with
//! `SUSI_HOME=C:/Users/user/.susi` — so every cloud session started with a
//! failed `susi-harness` server and a prompt to have the operator fix it on
//! their machine. A checked-in server names the clone's own build, relative
//! to the clone, and no host's home. The IDE configs checked in beside it
//! (VS Code, IntelliJ / Android Studio) held the same paths and are held to the
//! same rule. Until 2026-11-22 they also ran `susi mcp`, `susi workflow check`
//! and `susi tasks list`, which the rebuilt program does not have, so every
//! session started a failing server; each command they run must now be one the
//! program's own help lists.

/// The IDE configs that name the susi binary; each is read by every clone.
const IDE_CONFIGS: [&str; 3] = [
    ".vscode/settings.json",
    ".idea/mcp.json",
    ".idea/externalTools.xml",
];

/// Whether the program's help (`USAGE` in `src/main.rs`) lists `susi <words>`.
fn the_program_lists(words: &str) -> bool {
    let main = std::fs::read_to_string("src/main.rs").unwrap();
    let usage = main
        .split("const USAGE: &str = \"")
        .nth(1)
        .and_then(|rest| rest.split("\";").next())
        .unwrap();
    usage.contains(&format!("susi {words}"))
}

fn mcp_servers() -> serde_json::Map<String, serde_json::Value> {
    let text = std::fs::read_to_string(".mcp.json").unwrap();
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    json["mcpServers"].as_object().unwrap().clone()
}

/// A path only one machine has: absolute (`/…`, `\…`, `~…`, a drive letter)
/// or under a user's home directory.
fn names_a_machine_path(value: &str) -> bool {
    let bytes = value.as_bytes();
    let drive = bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
    let normalized = value.replace('\\', "/");
    drive
        || value.starts_with('/')
        || value.starts_with('\\')
        || value.starts_with('~')
        || normalized.contains("/Users/")
        || normalized.contains("/home/")
}

#[test]
fn checked_in_mcp_servers_name_no_machine_path() {
    let servers = mcp_servers();
    assert!(!servers.is_empty(), ".mcp.json declares no server");
    for (name, server) in &servers {
        let mut values: Vec<&str> = Vec::new();
        values.extend(server["command"].as_str());
        if let Some(args) = server["args"].as_array() {
            values.extend(args.iter().filter_map(|a| a.as_str()));
        }
        if let Some(env) = server["env"].as_object() {
            values.extend(env.values().filter_map(|v| v.as_str()));
        }
        for value in values {
            assert!(
                !names_a_machine_path(value),
                "{name} names one machine's path {value:?}: a cloud session (and every other clone) cannot reach it"
            );
        }
    }
}

#[test]
fn the_mcp_server_runs_the_clones_own_build_with_the_default_home() {
    let servers = mcp_servers();
    let server = &servers["susi-core"];
    assert_eq!(server["command"], "target/debug/susi");
    assert_eq!(server["args"], serde_json::json!(["mcp", "serve"]));
    assert!(
        the_program_lists("mcp serve"),
        "`susi mcp serve` left the program's help"
    );
    assert!(
        server["env"].get("SUSI_HOME").is_none(),
        "SUSI_HOME is the host's to set; ~/.susi is the default everywhere"
    );
}

#[test]
fn a_machine_path_is_told_from_a_clone_relative_one() {
    for path in [
        "F:/src/github.com/intellibitz/susi/target/debug/susi.exe",
        "C:/Users/user/.susi",
        "C:\\Users\\user\\.susi",
        "/home/user/susi/target/debug/susi",
        "~/.susi",
        "\\\\server\\share\\susi.exe",
    ] {
        assert!(names_a_machine_path(path), "{path}");
    }
    for path in ["target/debug/susi", "mcp", "scripts/susi-mcp.sh"] {
        assert!(!names_a_machine_path(path), "{path}");
    }
}

/// Every double-quoted value in `text`: JSON strings and XML attribute values
/// alike.
fn quoted_values(text: &str) -> Vec<&str> {
    text.split('"').skip(1).step_by(2).collect()
}

#[test]
fn checked_in_ide_configs_name_no_machine_path() {
    for file in IDE_CONFIGS {
        let text = std::fs::read_to_string(file).unwrap();
        for value in quoted_values(&text) {
            assert!(
                !names_a_machine_path(value),
                "{file} names one machine's path {value:?}: every other clone gets a dead path"
            );
        }
        assert!(
            !text.contains("SUSI_HOME"),
            "{file}: SUSI_HOME is the host's to set"
        );
    }
}

#[test]
fn ide_configs_run_the_clones_own_build() {
    let read = |file: &str| std::fs::read_to_string(file).unwrap();
    let vscode: serde_json::Value = serde_json::from_str(&read(".vscode/settings.json")).unwrap();
    assert_eq!(
        vscode["mcp.servers"]["susi-core"]["command"],
        "${workspaceFolder}/target/debug/susi"
    );
    assert_eq!(
        vscode["mcp.servers"]["susi-core"]["args"],
        serde_json::json!(["mcp", "serve"])
    );
    let idea: serde_json::Value = serde_json::from_str(&read(".idea/mcp.json")).unwrap();
    assert_eq!(
        idea["mcpServers"]["susi-core"]["command"],
        "target/debug/susi"
    );
    assert_eq!(
        idea["mcpServers"]["susi-core"]["args"],
        serde_json::json!(["mcp", "serve"])
    );
    let tools = read(".idea/externalTools.xml");
    let commands: Vec<&str> = tools
        .lines()
        .filter(|line| line.contains(r#"name="COMMAND""#))
        .collect();
    assert!(!commands.is_empty(), "externalTools.xml declares no tool");
    for line in &commands {
        assert!(
            line.contains(r#"value="$ProjectFileDir$/target/debug/susi""#),
            "{line}"
        );
    }
    let parameters: Vec<&str> = tools
        .lines()
        .filter(|line| line.contains(r#"name="PARAMETERS""#))
        .filter_map(|line| line.split("value=\"").nth(1)?.split('"').next())
        .collect();
    assert_eq!(
        parameters.len(),
        commands.len(),
        "every tool names its command"
    );
    for words in parameters {
        assert!(
            the_program_lists(words),
            "externalTools.xml runs `susi {words}`, which the program's help does not list"
        );
    }
}

#[test]
fn quoted_values_reads_json_strings_and_xml_attributes() {
    assert_eq!(
        quoted_values(r#"{"command": "F:/x.exe", "args": ["mcp"]}"#),
        ["command", "F:/x.exe", "args", "mcp"]
    );
    assert_eq!(
        quoted_values(r#"<option name="COMMAND" value="$ProjectFileDir$/target/debug/susi" />"#),
        ["COMMAND", "$ProjectFileDir$/target/debug/susi"]
    );
}
