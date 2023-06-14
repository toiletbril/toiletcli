//! A demo of `toiletcli`, `cat` program.

use std::{env::args, str::FromStr};
use std::fs::File;
use std::io::{stdout, BufReader, BufWriter, Error, Read, Write};
use std::process::ExitCode;

use toiletcli::flags::*;
use toiletcli::colors::*;
use toiletcli::common::*;

#[inline(always)]
fn color_output(color_code: &str) -> Result<(), Error> {
    let mut stdout = stdout();
    let color = Color::from_str(color_code)?;

    print!("{}", color.foreground());
    stdout.flush()?;

    Ok(())
}

fn cat(file_path: &String, color: &String) -> Result<(), Error> {
    let file = File::open(&file_path)?;

    let mut buf: [u8; 8192] = [0; 8192];
    let mut reader = BufReader::new(file);
    let mut writer = BufWriter::new(stdout());

    if !color.is_empty() {
        color_output(color)?;
    }

    while let Ok(count) = reader.read(&mut buf) {
        if count == 0 {
            break;
        }
        writer.write_all(&buf[..count])?;
    }

    writer.flush()?;

    return Ok(());
}

fn main() -> ExitCode {
    let mut _args = args();
    let program_name = name_from_path(&_args.next().expect("Path should be provided"));

    let mut show_help: bool;
    let mut color: String;

    let mut flags = toiletcli::flags!(
        color: StringFlag,   ["--color", "-c"],
        show_help: BoolFlag, ["--help"]
    );

    let args = parse_flags(&mut _args, &mut flags);

    if let Err(err) = args {
        eprintln!("{}: {}", program_name, err);
        return ExitCode::FAILURE;
    }

    let args = args.unwrap();

    if show_help {
        println!("USAGE: {} [-options] <file> [file2, file3, ...]", program_name);
        println!("Output a file to standart output. A demo for `toiletcli` crate.");
        println!("");
        println!("OPTIONS: -c, --color <color>\tColor output.");
        println!("             --help           \tDisplay this message.");
        return ExitCode::SUCCESS;
    }

    if args.len() < 1 {
        eprintln!(
            "{}: No path specified. Try '--help' for more information.",
            program_name
        );
        return ExitCode::FAILURE;
    }

    let mut code = ExitCode::SUCCESS;

    for arg in args.iter() {
        if let Err(err) = cat(arg, &color) {
            eprintln!("{}: {}", program_name, err);
            code = ExitCode::FAILURE;
        }
    }

    if !color.is_empty() {
        let _ = color_output("default");
    }

    code
}
