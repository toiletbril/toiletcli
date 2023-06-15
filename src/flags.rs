//! Command line flag parsing.

// TODO:
// - Don't allocate memory if flag was not specified?

/// Enum that contains value to be modified by `parse_flags`.
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
    /// `value` will always be the value from this flag's last occurence.
    StringFlag(&'a mut String),
    /// Will include all values specified, even if this flag was used multiple times.
    ManyFlag(&'a mut Vec<String>),
}

/// A pair with a value to be modified and flag aliases.
///
/// Short flags are two letter flags starting with one dash (`-n`).
/// Long flags are flags starting with two dashes (`--help`).
/// You can combine short `BoolFlag` flags, eg. `-vAsn` will set `true` to all `-v`, `-A`, `-s`, `-n` flags.
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
/// use toiletcli::flags::*;
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
    let mut result: Vec<String> = vec![];

    // Check flags in flag array for malformed flags in debug builds.
    if cfg!(debug_assertions) {
        for (_, flag_strings) in &*flags {
            for flag in flag_strings {
                assert!(!flag.contains(" "),
                        "Invalid flag: '{}'. Flag aliases should not contain spaces. EXAMPLE: '--help', '-h'", flag);
                assert!(flag.len() >= 2,
                        "Invalid flag '{}'. Flags are made of either a dash and a letter, like '-h' or two dashes with a word, like '--help'.", flag);
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
                                if first != None {
                                    return Err(format!("Flag '{}' requires a value and can't be combined.", flag));
                                }
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

        let args = parse_flags(&mut args_vector.into_iter(), &mut flags).unwrap();

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
        let _ = parse_flags(&mut args_vector.into_iter(), &mut flags).unwrap();
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn parse_flags_malformed_long() {
        let args_vector = vec!["program".to_string()];
        let mut malformed = false;
        let mut flags = vec![(FlagType::BoolFlag(&mut malformed), vec!["-malformed"])];
        let _ = parse_flags(&mut args_vector.into_iter(), &mut flags).unwrap();
    }
}
