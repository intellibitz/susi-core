#![forbid(unsafe_code)]
#![allow(
    clippy::panic,
    reason = "a test reports a missing page by failing, with the path it looked for"
)]

//! The ecosystem page, `ECOSYSTEM.md`, is the same byte for byte in Susi, Susi Core and Susi
//! Tools: documentation says what is true, and the world reads all three repositories. This
//! test and its twin in Susi hold each copy to one fingerprint, so the page changes in every
//! repository in the same step or a test fails. To change the page: edit it in all three
//! repositories, then set `PAGE_BYTES` and `PAGE_FNV1A64` below to what this test reports,
//! here and in Susi's twin alike.

/// Length of the shared page, with line endings written as LF.
const PAGE_BYTES: usize = 3088;
/// FNV-1a (64-bit) of the shared page, with line endings written as LF. A fingerprint for
/// sameness, not a security check.
const PAGE_FNV1A64: u64 = 0x0c3a_b706_44e8_b0fa;

fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

/// The file as LF text: a Windows checkout may write CRLF, and the page is the same page.
fn read_lf(name: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
    let raw = std::fs::read(&path)
        .unwrap_or_else(|e| panic!("{} must exist at the repository root: {e}", path.display()));
    let mut out = Vec::with_capacity(raw.len());
    let mut bytes = raw.iter().peekable();
    while let Some(byte) = bytes.next() {
        if *byte == b'\r' && bytes.peek() == Some(&&b'\n') {
            continue;
        }
        out.push(*byte);
    }
    out
}

#[test]
fn the_ecosystem_page_is_the_shared_one() {
    let page = read_lf("ECOSYSTEM.md");
    let found = (page.len(), fnv1a64(&page));
    assert_eq!(
        found,
        (PAGE_BYTES, PAGE_FNV1A64),
        "ECOSYSTEM.md is not the shared page ({} bytes, fnv1a64 {:#018x}). Change it in Susi, \
         Susi Core and Susi Tools together, then move both pins here and in Susi Core's twin.",
        found.0,
        found.1
    );
}

#[test]
fn the_readme_points_to_the_ecosystem_page() {
    let readme = String::from_utf8_lossy(&read_lf("README.md")).into_owned();
    assert!(
        readme.contains("(ECOSYSTEM.md)"),
        "README.md must link ECOSYSTEM.md, the page that says how the three repositories fit"
    );
}
