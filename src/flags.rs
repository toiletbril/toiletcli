//! Command line argument parsing.
//!
//! Long flags are made of two dashes and a word (`--help`).
//! Short flags are made of a dash and a single letter (`-n`).
//!
//! Short flags of [`BoolFlag`](type@FlagType::BoolFlag) type can be combined, eg. `-vAsn` will set `true` to all `-v`, `-A`, `-s`, `-n` flags.
//!
//! Flags which take a value can't be combined, and they don't use an equals punctuation (or similar) to separate the key and value.
//! E.g. no `-k=<value>`. The intended usage is `-k <value>`, with a key `-k` and a value of `<value>`.

use std::fmt;
use std::error::Error;

/// Enum that contains a mutable reference to be modified.
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
    /// Will change reference to `true` if present.
    BoolFlag(&'a mut bool),
    /// Requires a value.
    /// Will change reference to value passed after that flag if present.
    StringFlag(&'a mut String),
    /// Requires at least one value.
    /// Works the same as [`StringFlag`](type@FlagType::StringFlag), but will include all values if flag was used multiple times.
    ManyFlag(&'a mut Vec<String>),
    /// Will count the number of times a letter is repeated. Long version increases count by 1.
    RepeatFlag(&'a mut usize),
    /// Requires at least one value.
    /// Will include everything after that flag, treating all flags after that one as arguments.
    EverythingAfterFlag(&'a mut Vec<String>),
}

#[derive(Debug)]
pub enum FlagErrorType {
    CannotCombine,
    NoValueProvided,
    Unknown,
}

#[derive(Debug)]
pub struct FlagError {
    pub error_type: FlagErrorType,
    pub flag: String,
}

impl Error for FlagError {}

impl fmt::Display for FlagError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.error_type {
            FlagErrorType::CannotCombine   => write!(f, "Flag {} requires a value and can't be combined", self.flag),
            FlagErrorType::NoValueProvided => write!(f, "No value provided for {}", self.flag),
            FlagErrorType::Unknown         => write!(f, "Unknown flag {}", self.flag),
        }
    }
}

/// A pair with a reference to be modified and flag aliases.
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

/// Construct [`[Flag]`](type@Flag) variable more quickly.
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
    const SPACE_HELP: &str =
        "Flags should not contain whitespaces";
    const LEN_HELP: &str =
        "Flags should be made of either a single dash with one letter, like '-h', or two dashes with a word, like '--help'";
    const LONG_HELP: &str =
        "Long flags should start with two dashes, like '--help' or '--color'";
    const SHORT_HELP: &str =
        "Flags should start with '-' or '--', like '--help' or '-h'";

    for (_, flag_strings) in flags {
        for flag in flag_strings {
            assert!(!flag.contains(char::is_whitespace),
                "Invalid flag '{}'. {}", flag, SPACE_HELP);
            assert!(flag.len() >= 2,
                "Invalid flag '{}'. {}", flag, LEN_HELP);
            if flag.len() > 2 {
                assert!(flag.starts_with("--"),
                    "Invalid long flag: '{}'. {}", flag, LONG_HELP);
            } else {
                assert!(flag.starts_with('-'),
                    "Invalid flag: '{}'. {}", flag, SHORT_HELP);
            }
        }
    }
}

// -> Result<IsFlag, String>
fn parse_arg<Args>(arg: &String, args: &mut Args, flags: &mut [Flag]) -> Result<bool, FlagError>
where Args: Iterator<Item = String> {
    let mut chars = arg.chars().peekable();

    if chars.next() != Some('-') {
        return Ok(false);
    }

    let is_long = chars.peek() == Some(&'-');

    let mut found_long = false;
    let mut last_short_flag_with_value: Option<char> = None;

    for ch in chars {
        let mut found_short = false;

        for (flag_value, flag_strings) in &mut *flags {
            for flag in flag_strings {
                // Short flag is a string with a single dash and a character
                if flag.len() == 2 && flag.ends_with(ch) {
                    found_short = true;
                }
                // Long flag starts with 2 dashes, and should just match entirely
                else if ch == '-' && arg == *flag {
                    found_long = true;
                } else {
                    continue;
                }

                // Combining short flags that need a value is not allowed
                if let Some(first) = last_short_flag_with_value {
                    let error = FlagError {
                        error_type: FlagErrorType::CannotCombine,
                        flag: format!("-{}", first),
                    };
                    return Err(error);
                }

                match flag_value {
                    FlagType::BoolFlag(value) => {
                        **value = true;
                    }

                    FlagType::StringFlag(value) => {
                        if let Some(next_arg) = args.next() {
                            **value = next_arg.clone();
                            last_short_flag_with_value = Some(ch);
                        } else {
                            let error = FlagError {
                                error_type: FlagErrorType::NoValueProvided,
                                flag: flag.to_string(),
                            };
                            return Err(error);
                        }
                    }

                    FlagType::RepeatFlag(value) => {
                        **value += 1;
                    }

                    FlagType::ManyFlag(value) => {
                        if let Some(next_arg) = args.next() {
                            value.push(next_arg.clone());
                            last_short_flag_with_value = Some(ch);
                        } else {
                            let error = FlagError {
                                error_type: FlagErrorType::NoValueProvided,
                                flag: flag.to_string(),
                            };
                            return Err(error);
                        }
                    }

                    FlagType::EverythingAfterFlag(value) => {
                        for next_arg in args.by_ref() {
                            value.push(next_arg.clone());
                        }
                        if value.is_empty() {
                            let error = FlagError {
                                error_type: FlagErrorType::NoValueProvided,
                                flag: flag.to_string(),
                            };
                            return Err(error);
                        }
                    }
                }
            }
        }
        if found_long {
            break;
        } else if is_long && !found_long {
            let error = FlagError {
                error_type: FlagErrorType::Unknown,
                flag: arg.to_string()
            };
            return Err(error);
        } else if !found_short {
            let error = FlagError {
                error_type: FlagErrorType::Unknown,
                flag: format!("-{}", ch),
            };
            return Err(error);
        }
    }

    Ok(true)
}

/// Consumes and parses flags and arguments from [`Iterator<String>`](type@Iterator<String>).
///
/// # Returns
/// ## Ok
/// All arguments that are not flags. Can be empty.
/// Changes references passed in the enums according to parsed flags.
///
/// ## Err
/// - Unknown flag;
/// - No value provided for a flag that requires it;
/// - Short flag that takes a value was combined with other flag.
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
pub fn parse_flags<Args>(args: &mut Args, flags: &mut [Flag]) -> Result<Vec<String>, FlagError>
where Args: Iterator<Item = String> {
    let mut parsed_arguments: Vec<String> = vec![];

    #[cfg(debug_assertions)]
    check_flags(flags);

    while let Some(arg) = args.next() {
        let is_flag = parse_arg(&arg, args, flags)?;

        if !is_flag {
            parsed_arguments.push(arg);
        }
    }

    Ok(parsed_arguments)
}

/// Works the same way as [`parse_flags`](fn@parse_flags), but stops when it encounters the first argument.
/// Consumes all flags before the first argument from `args` iterator, so `args` can be used again to
/// parse the remaining contents. Will return `Ok("".to_string())` if no arguments were provided.
///
/// # Returns
/// ## Ok
/// First argument that is not a flag, or empty string when there is no arguments.
/// Changes references passed in the enums according to parsed flags.
///
/// ## Err
/// - Unknown flag;
/// - No value provided for a flag that requires it;
/// - Short flag that takes a value was combined with other flag.
///
/// ### Example
/// ```no_run
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
/// let subcommand = parse_flags_until_subcommand(&mut args, &mut main_flags).unwrap();
///
/// let mut d_flag;
///
/// let mut sub_flags = flags![
///     d_flag: BoolFlag, ["-d"]
/// ];
///
/// let subcommand_args = parse_flags(&mut args, &mut sub_flags);
/// ```
pub fn parse_flags_until_subcommand<Args>(args: &mut Args, flags: &mut [Flag]) -> Result<String, FlagError>
where Args: Iterator<Item = String> {
    #[cfg(debug_assertions)]
    check_flags(flags);

    while let Some(arg) = args.next() {
        let is_flag = parse_arg(&arg, args, flags)?;

        if !is_flag {
            return Ok(arg);
        }
    }

    Ok("".to_string())
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
    fn flag_everything_after() {
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
    fn flag_repeat_flag() {
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
    fn parse_flags_no_arguments() {
        let argv = vec!["program", "-v", "-d"];
        let mut args = argv.iter().map(|x| x.to_string());

        let program_name = args.next().unwrap();

        assert_eq!(program_name, "program".to_string());

        let mut v;
        let mut d;

        let mut flags = flags![
            v: BoolFlag, ["-v"],
            d: BoolFlag, ["-d"]
        ];

        let parsed_args = parse_flags(&mut args, &mut flags);

        assert_eq!(d, true);
        assert_eq!(v, true);
        assert!(parsed_args.unwrap().is_empty());
    }

    #[test]
    fn parse_flags_subcommand() {
        let argv = vec!["program", "-v", "dump", "-d", "argument"];
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
    fn parse_flags_subcommand_no_argument() {
        let argv = vec!["program", "-v", "-d"];
        let mut args = argv.iter().map(|x| x.to_string());

        let program_name = args.next().unwrap();

        assert_eq!(program_name, "program".to_string());

        let mut v;
        let mut d;

        let mut main_flags = flags![
            v: BoolFlag, ["-v"],
            d: BoolFlag, ["-d"]
        ];

        let subcommand = parse_flags_until_subcommand(&mut args, &mut main_flags);

        assert_eq!(v && d, true);
        assert!(subcommand.unwrap().is_empty());
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

        parse_flags(&mut args_vector.into_iter(), &mut flags).unwrap();
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

        parse_flags(&mut args_vector.into_iter(), &mut flags).unwrap();
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

        parse_flags(&mut args_vector.into_iter(), &mut flags).unwrap();
    }
}
