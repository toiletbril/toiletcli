//! A demo of `toiletcli`, `cat` program.

use std::env::args;
use std::fs::File;
use std::io::{stdout, BufReader, BufWriter, Read, Write};
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

fn cat(file_path: &String, color: &String) -> Result<(), String> {
    let result = File::open(&file_path);

    match result {
        Ok(file) => {
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
                if let Err(err) = writer.write_all(&buf[..count]) {
                    return Err(format!("{}: {}", file_path, err));
                }
            }

            if let Err(err) = writer.flush() {
                return Err(format!("{}: {}", file_path, err));
            }
        }
        Err(err) => {
            return Err(format!("{}: {}", file_path, err))
        }
    }

    return Ok(());
}

fn main() -> ExitCode {
    let mut _args = args();
    let program_name = toiletcli::name_from_path(&_args.next().expect("Path should be provided"));

    let mut color = String::new();
    let mut show_help = false;

    let mut flags = vec![
        (vec!["--colored", "-c"], FlagType::StringFlag(&mut color)),
        (vec!["--help"], FlagType::SimpleFlag(&mut show_help)),
    ];

    let args = toiletcli::parse_flags(&mut _args.collect(), &mut flags);

    if let Ok(args) = args {
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

        for arg in args.iter() {
            match cat(arg, &color) {
                Ok(()) => {}
                Err(err) => {
                    if !color.is_empty() {
                        color_output("default");
                    }
                    eprintln!("{}: {}", program_name, err);
                    return ExitCode::FAILURE;
                }
            }
        }
    } else {
        eprintln!("{}: {}", program_name, args.unwrap_err());
        return ExitCode::FAILURE;
    }

    if !color.is_empty() {
        color_output("default");
    }
    ExitCode::SUCCESS
}
