//! Unattended command launch helpers (`background_command`, `without_window`).
//!
//! Core ships for Linux only. Windows console-window creation flags lived here;
//! that ownership is in Susi Commercial Apps. `without_window` remains a no-op
//! so callers keep one launch path.

use std::ffi::OsStr;
use std::process::Command;

/// A [`Command`] for `program` that never opens a console window.
#[must_use]
pub fn background_command(program: impl AsRef<OsStr>) -> Command {
    let mut command = Command::new(program);
    without_window(&mut command);
    command
}

/// Make an existing `command` start without a console window.
///
/// On Linux this is a no-op: there is no console-window creation flag.
pub fn without_window(command: &mut Command) -> &mut Command {
    command
}

#[cfg(test)]
// Test-only assertions: a failed spawn or read must fail the test loudly.
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn a_background_command_runs_and_captures_output() {
        let mut command = background_command("sh");
        command.args(["-c", "echo hidden"]);
        let out = command.output().unwrap();
        assert!(out.status.success());
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "hidden");
    }
}
