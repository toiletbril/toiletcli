//! A demo of `toiletcli`, `cat` program.

use std::env::args;
use std::fs::File;
use std::io::{stdout, BufReader, BufWriter, Error, Read, Write};
use std::process::ExitCode;

use toiletcli::FlagType;

#[inline(always)]
fn color_output(color_code: &str) {
    match color_code.to_lowercase().as_str() {
        "default" => color_output("0"),
        "black"   => color_output("30"),
        "red"     => color_output("31"),
        "green"   => color_output("32"),
        "yellow"  => color_output("33"),
        "blue"    => color_output("34"),
        "purple"  => color_output("35"),
        "cyan"    => color_output("36"),
        "white"   => color_output("37"),
        _ => {
            let mut stdout = stdout();
            print!("\u{001b}[{}m", color_code);
            stdout.flush().expect("Should be able to flush");
        }
    }
}

fn cat(file_path: &String, color: &String) -> Result<(), Error> {
    let file = File::open(&file_path)?;

    let mut buf: [u8; 8192] = [0; 8192];
    let mut reader = BufReader::new(file);
    let mut writer = BufWriter::new(stdout());

    if !color.is_empty() {
        color_output(color);
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
    let program_name = toiletcli::name_from_path(&_args.next().expect("Path should be provided"));

    let mut show_help: bool;
    let mut color: String;

    let mut flags = toiletcli::flags!(
        color: StringFlag,   ["--colored", "-c"],
        show_help: BoolFlag, ["--help"]
    );

    let args = toiletcli::parse_flags(&mut _args.collect(), &mut flags);

    if let Err(err) = args {
        eprintln!("{}: {}", program_name, err);
        return ExitCode::FAILURE;
    }

    let args = args.unwrap();

    if show_help {
        println!("USAGE: {} [-options] <file> [file2, file3, ...]", program_name);
        println!("Output a file to standart output. A demo for `toiletcli` crate.");
        println!("");
        println!("OPTIONS: -c, --colored <color>\tColor output.");
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
        match cat(arg, &color) {
            Ok(()) => {}
            Err(err) => {
                eprintln!("{}: {}", program_name, err);
                code = ExitCode::FAILURE;
            }
        }
    }

    if !color.is_empty() {
        color_output("default");
    }

    code
}
