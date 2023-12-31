# toiletcli

A framework for command line applications, including a command line
argument parser.

Complete overview can be found in the documentation for each module.

Modules can be disabled/enabled via features:
```toml
[features]
default = ["flags", "colors", "escapes"]
```

## Examples

```rust
//! Command line argument parsing.

use std::env::args;

use toiletcli::flags;
use toiletcli::flags::{FlagType, parse_flags, parse_flags_until_subcommand};

let mut args = args();

// Most often, path to the program will be the first argument.
// This will prevent the function from parsing, as path to the program does not start with '-'.
let program_name = args.next().unwrap();

let mut v_flag;

let mut main_flags = flags![
    v_flag: BoolFlag, ["-v", "--long-v"]
];

let subcommand = parse_flags_until_subcommand(&mut args, &mut main_flags).unwrap();

let mut d_flag;

let mut sub_flags = flags![
    d_flag: BoolFlag, ["-d"]
];

let subcommand_args = parse_flags(&mut args, &mut sub_flags);
```

```rust
//! Convenient ANSI terminal colors and styles.

use toiletcli::colors::{Color, Style, UnderlineStyle, StyleBuilder};

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

```rust
//! Most common escapes to manipulate terminals.

use toiletcli::escapes::{Cursor, System};

println!("This is a 'word' that will be replaced!{}bird", Cursor::Position(12));
// This is a 'bird' that will be replaced!

println!("This is a '{}dog' that will be replaced too!{}cat", Cursor::Save, Cursor::Restore);
// This is a 'cat' that will be replaced too!

print!("{}", System::SetTitle("hello"));
// Look at the title :3
```

```rust
//! Common functions.

use toiletcli::common::{name_from_path};

let path = "toilet/bin/program";
let name = common::name_from_path(path);

assert_eq!(name, "program");
```
