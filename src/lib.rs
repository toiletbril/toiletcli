//! toiletcli is a collection of common functions that I use in my CLI applications.

/// Directory characters, which would be `"/\\"` on Windows and `"/"` on POSIX.
///
/// # Example
/// ```rust
/// use toiletcli::DIR_CHARS;
///
/// let path = "hello/world";
/// let mut program = String::new();
///
/// for (i, c) in path.chars().rev().enumerate() {
///     if DIR_CHARS.contains(c) {
///         program = String::from(path.split_at(path.len() - i).1);
///     }
/// }
///
/// assert_eq!(program, "world");
/// ```
pub const DIR_CHARS: &'static str = if cfg!(windows) { "\\/" } else { "/" };

/// Assertion that will be performed on compilation.
#[macro_export]
macro_rules! static_assert {
    ($cond:expr) => {
        #[allow(dead_code)]
        const fn static_assertion() {
            assert!($cond);
        }
        const _: () = static_assertion();
    };
}

/// Enum that contains value to be modified by `parse_flags`.
///
/// # Example
/// ```rust
/// use toiletcli::FlagType;
///
/// let mut color = String::new();
/// let mut show_help = false;
///
/// let mut flags = vec![
///     (FlagType::StringFlag(&mut color), vec!["--color", "-c"]),
///     (FlagType::BoolFlag(&mut show_help),   vec!["--help"], ),
/// ];
/// ```
#[derive(Debug, PartialEq)]
pub enum FlagType<'a> {
    /// Will change to `true` if present.
    BoolFlag(&'a mut bool),
    /// Will change to specified `value` if present, will remain unchanged if not.
    /// `value` will always be the value from this flag's last occurence.
    StringFlag(&'a mut String),
    /// Will include all values specified, even if this flag was used multiple times.
    ManyFlag(&'a mut Vec<String>)
}

/// Array of tuples for `parse_flags` with flag strings and value to be modified.
///
/// # Example
/// ```rust
/// use toiletcli::FlagType;
///
/// let mut color = String::new();
/// let mut show_help = false;
///
/// let mut flags = vec![
///     (FlagType::StringFlag(&mut color), vec!["--color", "-c"]),
///     (FlagType::BoolFlag(&mut show_help),   vec!["--help"], ),
/// ];
/// ```
pub type Flag<'a> = (FlagType<'a>, Vec<&'a str>, );

/// Construct `[Flag]` variable more quickly.
///
/// # Example
/// ```rust
/// use toiletcli::FlagType;
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
/// let mut flags_macro = toiletcli::flags!(
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

/// Parses CLI arguments from `sys::env::args`.
///
/// Short flags are two letter flags starting with one dash (`-n`).
/// Long flags are flags starting with two dashes (`--help`).
/// You can combine short `SimpleFlag` flags, eg. `-vAsn` will set `true` to all `-v`, `-A`, `-s`, `-n` flags.
/// **Short flags that use `StringFlag` can't be combined.**
/// # Returns
/// ## Ok
/// All arguments that are not flags.
/// Changes values passed in enums to matching values from args.
///
/// ## Side effects
/// ### `SimpleFlag`
/// ```rust
/// let value: bool = true;  // if flag is specified.
/// let value: bool = false; // if not.
/// ```
///
/// ### `StringFlag`
/// ```rust
/// let value: String = String::from("<value>"); // if flag is specified AND a value is provided,
/// let value: String = String::from("");        // if flag is not specified.
/// ```
///
/// ## Err
/// - On unknown flag;
/// - If `StringFlag` is specified, but no value is provided for it;
/// - If `StringFlag` is combined with some other flag.
///
/// # Example
/// ```rust
/// use std::env::args;
/// use toiletcli::FlagType;
///
/// let mut _args = args().collect();
///
/// let mut color = String::new();
/// let mut show_help = false;
///
/// let mut flags = vec![
///     (FlagType::StringFlag(&mut color),   vec!["--color", "-c"]),
///     (FlagType::BoolFlag(&mut show_help), vec!["--help"], ),
/// ];
///
/// let args = toiletcli::parse_flags(&_args, &mut flags);
/// ```
pub fn parse_flags(args: &Vec<String>, flags: &mut [Flag]) -> Result<Vec<String>, String> {
    let mut args = args.into_iter();
    let mut result: Vec<String> = vec![];
    result.reserve(args.len());

    // Check flags in flag array for malformed flags in debug builds.
    if cfg!(debug_assertions) {
        for (_, flag_strings) in &*flags {
            for flag in flag_strings {
                if flag.len() > 2 {
                    assert!(flag.starts_with("--"),
                            "Invalid long flag: '{}'. Long flags should start with '--'.\n
                             EXAMPLE: '--help', '--color'", flag);
                } else {
                    assert!(flag.starts_with("-") && flag.len() == 2,
                            "Invalid flag: '{}'. Flag should start with '-' or '--'.\n
                             EXAMPLE: '--help' (long flag), '-h' (short flag)", flag);
                }
            }
        }
    }

    while let Some(arg) = args.next() {
        let mut chars = arg.chars();

        if chars.next() != Some('-') {
            result.push(arg.clone());
            continue;
        }

        let mut found_long = false;
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
                        if arg == *flag {
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
                    if flag.len() == 2 && ch == flag.chars().last().unwrap() {
                        found = true;
                        match flag_value {
                            FlagType::BoolFlag(value) => {
                                if let Some(first) = first {
                                    return Err(format!("Flag '-{}' requires a value and can't be combined.", first));
                                }
                                **value = true;
                            }
                            FlagType::StringFlag(value) => {
                                if first != None {
                                    return Err(format!("Flag '{}' requires a value and can't be combined.", flag));
                                }
                                if let Some(next_arg) = args.next() {
                                    **value = next_arg.clone();
                                    first = Some(ch);
                                } else {
                                    return Err(format!("No value provided for '{}'", flag));
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
                        }
                    }
                }
            }

            if !found {
                return Err(format!("Unknown flag '-{}'", ch));
            }
        }
    }
    return Ok(result);
}

/// Gets file name from it's path.
///
/// # Example
/// ```rust
/// let _name = "toilet/bin/program";
/// let name = toiletcli::name_from_path(_name);
/// assert_eq!(name, "program");
/// ```
pub fn name_from_path(path: &str) -> String {
    for (i, c) in path.chars().rev().enumerate() {
        if DIR_CHARS.contains(c) {
            return String::from(path.split_at(path.len() - i).1);
        }
    }
    return String::from(path);
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
            (FlagType::BoolFlag(&mut show_help), vec!["--help"], ),
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
        let args_vector = vec![
            "program".to_string(),
            "argument_one".to_string(),
            "-aVns".to_string(),
            "--long-specific".to_string(),
            "something".to_string(),
            "--many".to_string(),
            "first".to_string(),
            "--many".to_string(),
            "second".to_string(),
            "argument_two".to_string(),
        ];

        let mut a = false;
        let mut big_v = false;
        let mut n = false;
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
            (FlagType::BoolFlag(&mut z), vec!["-z"]),
            (FlagType::ManyFlag(&mut many), vec!["--many"]),
        ];

        let args = parse_flags(&args_vector, &mut flags).unwrap();

        assert_eq!(
            args, vec!["program", "argument_one", "argument_two"],
            "args: {:?} should be: {:?}",
            args, vec!["program", "argument_one", "argument_two"]
        );
        assert_eq!(
            (
                a, big_v, n, s,
                &long_specific, &not_used, z,
                &many
            ),
            (
                true, true, true, true,
                &"something".to_string(), &"".to_string(), false,
                &vec!["first".to_string(), "second".to_string()]
            ),
            "flags: {:?} should be: {:?}",
            (
                a, big_v, n, s,
                &long_specific, &not_used, z,
                &many
            ),
            (
                true, true, true, true,
                &"something".to_string(), &"".to_string(), false,
                &vec!["first".to_string(), "second".to_string()]
            ),
        );
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn parse_flags_malformed() {
        let args_vector = vec!["program".to_string()];

        let mut malformed = false;

        let mut flags = vec![(FlagType::BoolFlag(&mut malformed), vec!["m"])];

        let _ = parse_flags(&args_vector, &mut flags).unwrap();
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn parse_flags_malformed_long() {
        let args_vector = vec!["program".to_string()];

        let mut malformed = false;

        let mut flags = vec![(FlagType::BoolFlag(&mut malformed), vec!["-malformed"])];

        let _ = parse_flags(&args_vector, &mut flags).unwrap();
    }

    #[test]
    fn program_name() {
        let _name = if cfg!(windows) {
            "toilet\\bin\\program.exe"
        } else {
            "toilet/bin/program"
        };
        let name = name_from_path(_name);
        assert_eq!(
            name,
            if cfg!(windows) {
                "program.exe"
            } else {
                "program"
            }
        );
    }
}
