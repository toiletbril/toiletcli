# toiletcli

A collection of common functions that I use in my CLI applications.
This is another I-use-the-language-for-the-first-time repo (and I had a lot of fun with Rust so far).

## `pub mod flags`;
```rust
//! Utilities for command line flags parsing.

use std::env::args;

use toiletcli::flags;
use toiletcli::flags::*;

let mut color: String;
let mut show_help: bool;

let mut flags = flags!(
    color: StringFlag,   ["--color", "-c"],
    show_help: BoolFlag, ["--help"]
);

let args = parse_flags(&mut args(), &mut flags);
```

## `pub mod colors`;

```rust
//! Tools for ASCII terminal colors.
//! Contains enums that implement `Display` trait.

use toiletcli::colors::Color;
use toiletcli::colors::PrintableColor;

println!("{}{}This is red text on blue background!", Color::Red, Color::Blue.background());
```

## `pub mod common`;
```rust
//! Common modules and functions.

use toiletcli::common;

let path = "toilet/bin/program";
let name = common::name_from_path(path);

assert_eq!(name, "program");
```
