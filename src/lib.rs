//! Minimal framework for command line applications.
//!
//! This crate contains examples for each module and a demo `cat` program, which can be built/run with:
//! ```console
//! $ cargo run --example <cat/flags/colors>
//! ```

pub mod common;
#[cfg(feature = "flags")]
pub mod flags;
#[cfg(feature = "colors")]
pub mod colors;
#[cfg(feature = "escapes")]
pub mod escapes;
