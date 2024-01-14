//! `flags` module usage showcase.

use std::env::args;

use toiletcli::flags;
use toiletcli::flags::*;

fn main() -> () {
    let mut files;
    let mut value;
    let mut small;
    let mut big;
    let mut repeat;
    let mut with_dash;

    let mut flags = flags!(
        files: ManyFlag,       ["-f", "--file"],
        big: BoolFlag,         ["-b", "--big"],
        small: BoolFlag,       ["-s", "--small"],
        repeat: RepeatFlag,    ["-r", "--repeat"],
        value: StringFlag,     ["--value"],
        with_dash: StringFlag, ["--with-dash"]
    );

    let args = parse_flags(&mut args(), &mut flags);

    if let Err(err) = args {
        println!("Parsing Error: {}", err);
        return;
    }

    let args = args.unwrap();

    println!("Parsed flags: {:?}", flags);
    println!("Arguments: {:?}", args);
}
