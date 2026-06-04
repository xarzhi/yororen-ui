//! `yororen-ui-demos` — meta-crate index for the 13 demo crates.
//!
//! Each demo is its own crate under `crates/yororen-ui-demos/`,
//! with its own `Cargo.toml` and `main.rs`. This meta-crate
//! doesn't currently re-export them as Rust modules (each demo
//! is a binary-only crate, not a library), but it gives
//! downstream apps a single workspace member to depend on when
//! they want a manual QA bundle.
//!
//! To run a demo:
//!
//! ```sh
//! cargo run -p counter-demo
//! cargo run -p theme-showcase-demo
//! ```
//!
//! See the workspace `Cargo.toml` `members` list for the
//! canonical set of demo crates.

pub use yororen_ui as core;
