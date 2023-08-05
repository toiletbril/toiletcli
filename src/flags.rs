//! Command line flag parsing.

/// Enum that contains value to be modified.
///
/// # Example
/// ```rust
/// use toiletcli::flags::FlagType;
///
/// let mut color = String::new();
/// let mut show_help = false;
///
/// let mut flags = vec![
///     (FlagType::StringFlag(&mut color),   vec!["--color", "-c"]),
///     (FlagType::BoolFlag(&mut show_help), vec!["--help"]),
/// ];
/// ```
#[derive(Debug, PartialEq)]
pub enum FlagType<'a> {
    /// Will change to `true` if present.
    BoolFlag(&'a mut bool),
    /// Will change to specified `value` if present, will remain unchanged if not.
    /// `value` will always be the value from that flag's last occurence.
    StringFlag(&'a mut String),
    /// Will include all values specified, even if that flag was used multiple times.
    ManyFlag(&'a mut Vec<String>),
    /// Flag that remembers repeat count of letter. Long version will return 1.
    RepeatFlag(&'a mut usize),
    /// Will include everything after that flag.
    EverythingAfterFlag(&'a mut Vec<String>),
}

/// A pair with a value to be modified and flag aliases.
///
/// Short flags are two letter flags starting with one dash (`-n`).
/// Long flags are flags starting with two dashes (`--help`).
/// You can combine short `BoolFlag` flags, eg. `-vAsn` will set `true` to all `-v`, `-A`, `-s`, `-n` flags.
/// This is deliberately made that way to the detriment of parsing to avoid verbosity when declaring flags.
///
/// # Example
/// ```rust
/// use toiletcli::flags::FlagType;
///
/// let mut color = String::new();
/// let mut show_help = false;
///
/// let mut flags = vec![
///     (FlagType::StringFlag(&mut color),   vec!["--color", "-c"]),
///     (FlagType::BoolFlag(&mut show_help), vec!["--help"]),
/// ];
/// ```
pub type Flag<'a> = (FlagType<'a>, Vec<&'a str>, );

/// Construct `[Flag]` variable more quickly.
///
/// # Example
/// ```rust
/// use toiletcli::flags;
/// use toiletcli::flags::FlagType;
///
/// let mut color = String::new();
/// let mut show_help = false;
///
/// let mut flags = vec![
///     (FlagType::StringFlag(&mut color),   vec!["--color", "-c"]),
///     (FlagType::BoolFlag(&mut show_help), vec!["--help"], ),
/// ];
///
/// let mut color_macro;
/// let mut show_help_macro;
///
/// let mut flags_macro = flags!(
///     color_macro: StringFlag,   ["--color", "-c"],
///     show_help_macro: BoolFlag, ["--help"]
/// );
///
/// assert_eq!(flags, flags_macro);
/// ```
#[macro_export]
macro_rules! flags {
    ($($name:ident: $ty:ident, [$($strings:tt)*]),*) => {
        {
            $($name = Default::default();)*
            let mut flags = vec![];
            $(flags.push((FlagType::$ty(&mut $name), vec![$($strings)*]));)*
            flags
        }
    };
}

// Check flags in flag array for malformed flags in debug builds.
#[cfg(debug_assertions)]
fn check_flags(flags: &[Flag]) {
    for (_, flag_strings) in &*flags {
        for flag in flag_strings {
            assert!(!flag.contains(" "),
            "Invalid flag: '{}'. Flag aliases should not contain spaces. EXAMPLE: '--help', '-h'", flag);
            assert!(flag.len() >= 2,
            "Invalid flag '{}'. Flags are made of either a dash and a letter, like '-h' or two dashes with a word, like '--help'", flag);
            if flag.len() > 2 {
                assert!(flag.starts_with("--"),
                "Invalid long flag: '{}'. Long flags should start with '--'.\nEXAMPLE: '--help', '--color'", flag);
            } else {
                assert!(flag.starts_with("-"),
                "Invalid flag: '{}'. Flag should start with '-' or '--'. EXAMPLE: '--help' (long flag), '-h' (short flag)", flag);
            }
        }
    }
}

// -> Result<IsFlag, String>
fn parse_arg<Args>(arg: &String, args: &mut Args, flags: &mut [Flag]) -> Result<bool, String>
where Args: Iterator<Item = String> {
    let mut chars = arg.chars();

    if chars.next() != Some('-') {
        return Ok(false);
    }

    let mut found_long = false;
    let mut in_repeat = None;
    let mut first = None;

    while let Some(ch) = chars.next() {
        let mut found = false;

        if found_long {
            break;
        }

        // Longs flags go here.
        if ch == '-' {
            for (flag_value, flag_strings) in &mut *flags {
                for flag in flag_strings {
                    if arg != *flag {
                        continue;
                    }

                    found_long = true;

                    match flag_value {
                        FlagType::BoolFlag(value) => {
                            **value = true;
                        }

                        FlagType::StringFlag(value) => {
                            if let Some(next_arg) = args.next() {
                                **value = next_arg.clone();
                            } else {
                                return Err(format!("No value provided for '{}'", flag));
                            }
                        }

                        FlagType::ManyFlag(value) => {
                            if let Some(next_arg) = args.next() {
                                value.push(next_arg.clone());
                            } else {
                                return Err(format!("No value provided for '{}'", flag));
                            }
                        }

                        FlagType::RepeatFlag(value) => {
                            **value = 1;
                        }

                        FlagType::EverythingAfterFlag(value) => {
                            while let Some(next_arg) = args.next() {
                                value.push(next_arg.clone());
                            }
                            if value.is_empty() {
                                return Err(format!("No value provided for '{}'", flag));
                            }
                        }
                    }
                }
            }
            if !found_long {
                return Err(format!("Unknown long flag '{}'", arg));
            }
            break;
        }

        // One letter flags go here. (eg. -aVsd)
        for (flag_value, flag_strings) in &mut *flags {
            for flag in flag_strings {
                if flag.len() != 2 {
                    continue;
                }

                let short_flag = flag.chars().last().unwrap();

                if ch != short_flag {
                    continue;
                }

                found = true;

                // Don't allow combining short flags that have a value.
                // Combining flags without value with a single value flag is allowed.
                if let Some(first) = first {
                    return Err(format!("Flag '-{}' requires a value and can't be combined", first));
                }

                match flag_value {
                    FlagType::BoolFlag(value) => {
                        **value = true;
                    }

                    FlagType::StringFlag(value) => {
                        if let Some(next_arg) = args.next() {
                            **value = next_arg.clone();
                            first = Some(ch);
                        } else {
                            return Err(format!("No value provided for '{}'", flag));
                        }
                    }

                    FlagType::RepeatFlag(value) => {
                        if in_repeat == None || in_repeat == Some(ch) {
                            **value += 1;
                        } else {
                            in_repeat = Some(ch);
                            **value = 1;
                        }
                    }

                    FlagType::ManyFlag(value) => {
                        if let Some(next_arg) = args.next() {
                            value.push(next_arg.clone());
                            first = Some(ch);
                        } else {
                            return Err(format!("No value provided for '{}'", flag));
                        }
                    }

                    FlagType::EverythingAfterFlag(value) => {
                        while let Some(next_arg) = args.next() {
                            value.push(next_arg.clone());
                        }

                        if value.is_empty() {
                            return Err(format!("No value provided for '{}'", flag));
                        }
                    }
                }
            }
        }

        if !found {
            return Err(format!("Unknown flag '-{}'", ch));
        }
    }

    Ok(true)
}

/// Consumes and parses CLI arguments from `Iterator<String>`.
///
/// # Returns
/// ## Ok
/// All arguments that are not flags.
/// Changes values passed in enums to matching values from args.
///
/// ## Err
/// - On unknown flag;
/// - If `StringFlag` or `ManyFlag` is specified, but no value is provided for it;
/// - If short `StringFlag` or `ManyFlag` is combined with some other flag.
///
/// # Example
/// ```rust
/// use std::env::args;
/// use toiletcli::flags;
/// use toiletcli::flags::{FlagType, parse_flags};
///
/// let mut color = String::new();
/// let mut show_help = false;
///
/// let mut flags = flags!(
///     color: StringFlag,   ["--color", "-c"],
///     show_help: BoolFlag, ["--help"]
/// );
///
/// let args = parse_flags(&mut args(), &mut flags);
/// ```
pub fn parse_flags<Args>(args: &mut Args, flags: &mut [Flag]) -> Result<Vec<String>, String>
where Args: Iterator<Item = String> {
    let mut parsed_arguments: Vec<String> = vec![];

    if cfg!(debug_assertions) {
        check_flags(&flags);
    }

    while let Some(arg) = args.next() {
        let is_flag = parse_arg(&arg, args, flags)?;

        if !is_flag {
            parsed_arguments.push(arg.clone());
        }
    }

    Ok(parsed_arguments)
}

/// Works the same way as `parse_args`, but stops when it encounters the first argument.
/// Consumes everything before first arguments from `args` iterator, so `args` can be used again to
/// parse the remaining arguments for a subcommand.
///
/// # Returns
/// ## Ok
/// First argument that is not a flag.
/// Changes values passed in enums to matching values from args.
///
/// ## Err
/// - On unknown flag;
/// - If `StringFlag` or `ManyFlag` is specified, but no value is provided for it;
/// - If short `StringFlag` or `ManyFlag` is combined with some other flag.
/// 
/// ### Example
/// ```rust
/// use std::env::args;
/// use toiletcli::flags;
/// use toiletcli::flags::{FlagType, parse_flags, parse_flags_until_subcommand};
/// 
/// let mut args = args();
/// 
/// // Most often, path to the program will be the first argument.
/// // This will prevent the function from parsing, as path to the program does not start with '-'.
/// let program_name = args.next().unwrap();
///
/// let mut v_flag;
///
/// let mut main_flags = flags![
///     v_flag: BoolFlag, ["-v"]
/// ];
///
/// let subcommand = parse_flags_until_subcommand(&mut args, &mut main_flags);
/// 
/// let mut d_flag;
///
/// let mut sub_flags = flags![
///     d_flag: BoolFlag, ["-d"]
/// ];
///
/// let subcommand_args = parse_flags(&mut args, &mut sub_flags);
/// ```
pub fn parse_flags_until_subcommand<Args>(args: &mut Args, flags: &mut [Flag]) -> Result<String, String>
where Args: Iterator<Item = String> {
    let mut subcommand = String::new();

    if cfg!(debug_assertions) {
        check_flags(&flags);
    }

    while let Some(arg) = args.next() {
        let is_flag = parse_arg(&arg, args, flags)?;

        if !is_flag {
            subcommand = arg.clone();
            break;
        }
    }

    Ok(subcommand)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_macro() {
        let mut color = String::new();
        let mut show_help = false;

        let flags = vec![
            (FlagType::StringFlag(&mut color),   vec!["--color", "-c"]),
            (FlagType::BoolFlag(&mut show_help), vec!["--help"]),
        ];

        let mut color_macro;
        let mut show_help_macro;

        let flags_macro = flags!(
            color_macro: StringFlag,   ["--color", "-c"],
            show_help_macro: BoolFlag, ["--help"]
        );

        assert_eq!(flags, flags_macro);
    }

    #[test]
    fn parse_flags_default() {
        let argv = vec![
            "program",
            "argument_one",
            "-aVns",
            "--long-specific",
            "something",
            "-vvvvv",
            "--many",
            "first",
            "--many",
            "second",
            "argument_two",
        ];
        let mut args = argv.iter().map(|x| x.to_string());

        let mut a = false;
        let mut big_v = false;
        let mut n = false;
        let mut v = 0;
        let mut s = false;

        let mut long_specific = String::new();

        let mut z = false;
        let mut not_used = String::new();

        let mut many = vec![];

        let mut flags = vec![
            (FlagType::BoolFlag(&mut a), vec!["-a"]),
            (FlagType::BoolFlag(&mut big_v), vec!["-V"]),
            (FlagType::BoolFlag(&mut n), vec!["-n"]),
            (FlagType::BoolFlag(&mut s), vec!["-s"]),
            (FlagType::StringFlag(&mut long_specific), vec!["--long-specific"]),
            (FlagType::StringFlag(&mut not_used), vec!["--not-used"]),
            (FlagType::RepeatFlag(&mut v), vec!["-v"]),
            (FlagType::BoolFlag(&mut z), vec!["-z"]),
            (FlagType::ManyFlag(&mut many), vec!["--many"]),
        ];

        let parsed_args = parse_flags(&mut args, &mut flags).unwrap();

        assert_eq!(parsed_args, vec!["program", "argument_one", "argument_two"]);
        assert_eq!(a && big_v && n && s, true);
        assert_eq!(v, 5);
        assert_eq!(z, false);
        assert_eq!(long_specific, "something");
        assert_eq!(not_used, "");
        assert_eq!(many, vec!["first", "second"])
    }

    #[test]
    fn parse_everyting_after() {
        let argv = vec!["program", "-v", "-rr", "-e", "argument", "-file", "hello!", "-rrrr"];
        let mut args = argv.iter().map(|x| x.to_string());

        let mut v;
        let mut r;
        let mut everything_after;

        let mut flags = flags![
            v: RepeatFlag, ["-v"],
            r: RepeatFlag, ["-r"],
            everything_after: EverythingAfterFlag, ["-e"]
        ];

        let parsed_args = parse_flags(&mut args, &mut flags);

        assert_eq!(v, 1);
        assert_eq!(r, 2);
        assert_eq!(everything_after, vec!["argument", "-file", "hello!", "-rrrr"]);
        assert_eq!(parsed_args.unwrap(), vec!["program"]);
    }

    #[test]
    fn parse_repeat_flag() {
        let argv = vec!["program", "-vvvv", "-rrr", "--test", "argument"];
        let mut args = argv.iter().map(|x| x.to_string());

        let mut v;
        let mut r;
        let mut t;
        let mut unused;

        let mut flags = flags![
            v: RepeatFlag,      ["-v"],
            r: RepeatFlag,      ["-r"],
            unused: RepeatFlag, ["-u", "--unused"],
            t: RepeatFlag,      ["-t", "--test"]
        ];

        let parsed_args = parse_flags(&mut args, &mut flags);

        assert_eq!(v, 4);
        assert_eq!(r, 3);
        assert_eq!(unused, 0);
        assert_eq!(t, 1);
        assert_eq!(parsed_args.unwrap(), vec!["program", "argument"]);
    }

    #[test]
    fn parse_subcommands() {
        let argv = vec!["program", "-v","dump", "-d", "argument"];
        let mut args = argv.iter().map(|x| x.to_string());
         
        let program_name = args.next().unwrap();
        
        assert_eq!(program_name, "program".to_string());

        let mut v;

        let mut main_flags = flags![
            v: BoolFlag, ["-v"]
        ];

        let subcommand = parse_flags_until_subcommand(&mut args, &mut main_flags);

        assert_eq!(v, true);
        assert_eq!(subcommand.unwrap(), "dump".to_string());

        let mut d;

        let mut sub_flags = flags![
            d: BoolFlag, ["-d"]
        ];

        let parsed_args = parse_flags(&mut args, &mut sub_flags);

        assert_eq!(d, true);
        assert_eq!(parsed_args.unwrap(), vec!["argument"]);
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn parse_flags_malformed() {
        let args_vector = vec!["program".to_string()];

        let mut malformed = false;
        
        let mut flags = vec![
            (FlagType::BoolFlag(&mut malformed), vec!["m"])
        ];
    
        let _ = parse_flags(&mut args_vector.into_iter(), &mut flags).unwrap();
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn parse_flags_malformed_long() {
        let args_vector = vec!["program".to_string()];
        
        let mut malformed = false;
        
        let mut flags = vec![
            (FlagType::BoolFlag(&mut malformed), vec!["-onedash"])
        ];
        
        let _ = parse_flags(&mut args_vector.into_iter(), &mut flags).unwrap();
    }

    
    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn parse_flags_malformed_space() {
        let args_vector = vec!["program".to_string()];

        let mut malformed = false;

        let mut flags = vec![
            (FlagType::BoolFlag(&mut malformed), vec!["--space bar"])
        ];

        let _ = parse_flags(&mut args_vector.into_iter(), &mut flags).unwrap();
    }
}
