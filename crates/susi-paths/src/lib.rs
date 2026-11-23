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

//! The platform-service table Susi fills on Windows ([`platform`]) and windowless command
//! launch ([`console`]).

pub mod console;
pub mod platform;
