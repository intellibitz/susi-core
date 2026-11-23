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

//! The append-only, owner-only JSONL metrics sink ([`sink`]) Susi's Windows crate writes
//! through.

pub mod sink;
