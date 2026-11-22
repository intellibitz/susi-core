#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]

//! Shared code has exactly one checked-in implementation. These tests make
//! physical copies a regression: a consumer reaches shared code through a
//! Cargo edge on the owning crate (see `architecture_tests.rs` for the leaf
//! order and the cross-crate `#[path]` mount ratchet).

use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

#[test]
fn checked_in_rust_implementations_are_not_byte_duplicates() {
    let root = workspace_root();
    // `xtask/` moved to Susi Commercial Apps (`tools/core-xtask`); only scan
    // directories that still exist in the slim core checkout.
    let mut pending: Vec<PathBuf> = ["crates", "src", "tests", "toolchain"]
        .into_iter()
        .map(|name| root.join(name))
        .filter(|path| path.is_dir())
        .collect();
    let mut unique = std::collections::HashMap::<Vec<u8>, PathBuf>::new();
    let mut duplicates = Vec::new();

    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("read source directory") {
            let path = entry.expect("read source entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
                let source = std::fs::read(&path).expect("read Rust source");
                if let Some(first) = unique.insert(source, path.clone()) {
                    duplicates.push((first, path));
                }
            }
        }
    }

    assert!(
        duplicates.is_empty(),
        "byte-identical Rust files must be consolidated into one owning crate: {duplicates:?}"
    );
}
