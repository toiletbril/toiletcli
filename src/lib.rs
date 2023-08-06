//! # toiletcli
//!
//! A tiny framework for command line applications, including a command line
//! argument parser.
//!
//! Complete overview can be found in the documentation for each module.

pub mod common;
#[cfg(feature = "flags")]
pub mod flags;
#[cfg(feature = "colors")]
pub mod colors;
#[cfg(feature = "escapes")]
pub mod escapes;
