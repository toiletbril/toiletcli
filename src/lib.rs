//! # toiletcli
//!
//! A tiny framework for command line applications, including a command line
//! argument parser.
//!
//! Complete overview can be found in the documentation for each module.

#[cfg(feature = "colors")]
pub mod colors;
pub mod common;
#[cfg(feature = "escapes")]
pub mod escapes;
#[cfg(feature = "flags")]
pub mod flags;
