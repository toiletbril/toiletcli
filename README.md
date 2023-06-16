# toiletcli

Light framework for command line applications.
This is I-use-the-language-for-the-first-time project (I had a lot of fun with Rust so far).

This crate contains examples for each module and a demo `cat` program, which can be built/run with:
```console
$ cargo run --example <cat/flags/colors>
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

## `pub mod colors;`

```rust
//! ANSI terminal colors as enums that all implement `Display` and `FromStr` traits.

use toiletcli::colors::Color;
use toiletcli::colors::PrintableColor;

println!("{}{}This is red text on blue background!{}",
         Color::Red, Color::Blue.bg(), Style::Reset);

let weird_style = StyleBuilder::new()
    .foreground(Color::Byte(93))
    .background(Color::from_str("black").unwrap())
    .add_style(Style::Underlined)
    .underline_color(Color::RGB(0, 255, 0))
    .underline_style(UnderlineStyle::Curly)
    .build();

println!("{}RGB purple on black background with RGB curly green underline!{}",
        weird_style, Style::Reset);
```

## `pub mod lines;`

```rust
//! Enums for ANSI terminal cursor manipulation that all implement `Display`.

use toiletcli::lines::*;

println!("This is a 'word' that will be replaced!{}bird", Cursor::Position(12));
// This is a 'bird' that will be replaced!

println!("This is a '{}dog' that will be replaced too!{}cat", Cursor::Save, Cursor::Restore);
// This is a 'cat' that will be replaced too!
```

## `pub mod common;`
```rust
//! Common functions.

use toiletcli::common;

let path = "toilet/bin/program";
let name = common::name_from_path(path);

assert_eq!(name, "program");
```
