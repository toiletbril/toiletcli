//! `flags` module usage showcase.

use std::env::args;

use toiletcli::flags;
use toiletcli::flags::*;

fn main() -> () {
    let mut files;
    let mut value;
    let mut small;
    let mut big;

    let mut flags = flags!(
        files: ManyFlag,   ["--file", "-f"],
        value: StringFlag, ["--value"],
        big: BoolFlag,     ["--big", "-b"],
        small: BoolFlag,   ["--small", "-s"]
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
