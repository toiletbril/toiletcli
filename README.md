# toiletcli

Minimal framework for command line applications.
This is I-use-the-language-for-the-first-time project (I had a lot of fun with Rust so far).

This crate contains examples and a demo `cat` program, which can be built/run with:
```console
$ cargo run --example <...>
```

## `pub mod flags;`

```rust
//! Command line flag parsing.

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

## `pub mod common;`
```rust
//! Common functions.

use toiletcli::common;

let path = "toilet/bin/program";
let name = common::name_from_path(path);

assert_eq!(name, "program");
```
