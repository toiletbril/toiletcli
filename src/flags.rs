//! Command line argument parsing.
//!
//! Long flags are made of two dashes and a word (`--help`).
//! Short flags are made of a dash and a single letter (`-n`).
//!
//! Short flags of [`BoolFlag`](type@FlagType::BoolFlag) type can be combined,
//! eg. `-vAsn` will set `true` to all `-v`, `-A`, `-s`, `-n` flags.
//!
//! Flags which take a value can't be combined.
//!
//! The intended usage of flags which take a value is `-k <value>`/`-k=<value>`,
//! with a key `-k` and a value of `<value>`.
//!
//! When parsing whole input, a special flag `--` will cause the rest of the
//! input to be treated as arguments, ignoring the `--` itself. When parsing
//! only until a subcommand, `--` will be treated as an argument.

use std::error::Error;
use std::fmt;

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
pub enum FlagType<'a>
{
  /// Will change reference to `true` if present.
  BoolFlag(&'a mut bool),
  /// Requires a value.
  /// Will change reference to value passed after that flag if present.
  StringFlag(&'a mut String),
  /// Requires at least one value.
  /// Works the same as [`StringFlag`](type@FlagType::StringFlag), but will
  /// include all values if flag was used multiple times.
  ManyFlag(&'a mut Vec<String>),
  /// Will count the number of times a letter is repeated. Long version
  /// increases count by 1.
  RepeatFlag(&'a mut usize),
}

#[derive(Debug)]
pub enum FlagErrorType
{
  CannotCombine,
  NoValueProvided,
  ExtraValueProvided,
  Unknown,
}

#[derive(Debug)]
pub struct FlagError
{
  pub error_type: FlagErrorType,
  /// Contains the flag that caused this error.
  pub flag: String,
}

impl Error for FlagError {}

impl fmt::Display for FlagError
{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
  {
    match self.error_type {
      FlagErrorType::CannotCombine => {
        write!(f, "Flag {} requires a value and can't be combined", self.flag)
      }
      FlagErrorType::NoValueProvided => {
        write!(f, "No value provided for {}", self.flag)
      }
      FlagErrorType::ExtraValueProvided => {
        write!(f, "Flag {} does not take a value", self.flag)
      }
      FlagErrorType::Unknown => write!(f, "Unknown flag {}", self.flag),
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
pub type Flag<'a> = (FlagType<'a>, Vec<&'a str>);

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
fn check_flags(flags: &[Flag])
{
  const SPACE_HELP: &str = "Flags should not contain whitespace.";
  const LEN_HELP: &str =
    "Flags should be made of either a single dash with one letter, like \
        '-h', or two dashes with a word, like '--help'.";
  const LONG_HELP: &str =
    "Long flags should start with two dashes, like '--help' or '--color'.";
  const SHORT_HELP: &str =
    "Flags should start with '-' or '--', like '--help' or '-h'.";

  for (_, flag_strings) in flags {
    for flag in flag_strings {
      assert!(!flag.contains(char::is_whitespace),
              "Invalid flag '{}'. {}",
              flag,
              SPACE_HELP);
      assert!(flag.len() >= 2, "Invalid flag '{}'. {}", flag, LEN_HELP);
      if flag.len() > 2 {
        assert!(flag.starts_with("--"),
                "Invalid long flag: '{}'. {}",
                flag,
                LONG_HELP);
      } else {
        assert!(flag.starts_with('-'),
                "Invalid flag: '{}'. {}",
                flag,
                SHORT_HELP);
      }
    }
  }
}

fn parse_arg<Args>(arg: &String,
                   args: &mut Args,
                   flags: &mut [Flag])
                   -> Result<bool, FlagError>
  where Args: Iterator<Item = String>
{
  // Split flags that look like `-k=value` to key and value. Otherwise we'll use
  // the next argument from the iterator as key value.
  let (arg_flag, arg_val): (&str, Option<&str>) =
    if let Some(pos) = arg.find('=') {
      let (f, v) = arg.split_at(pos);
      (f, Some(v.strip_prefix('=').expect("unreachable")))
    } else {
      (arg, None)
    };

  // Decide whether arg is a long flag or short flag. We want chars to start
  // with letters, with all dashed skipped over.
  let mut arg_chars = arg_flag.chars().peekable();
  if arg_chars.next() != Some('-') {
    return Ok(false);
  }
  let is_long = if arg_chars.peek() == Some(&'-') {
    arg_chars.next();
    true
  } else {
    false
  };

  let mut found_long = false;
  let mut last_short_flag_with_value: Option<char> = None;

  // This iterates the characters of the arg, in case this arg consists of
  // several short flags. If this is a long flag, we'll just break out after the
  // first loop.
  for ch in arg_chars {
    let mut found_short = false;

    // Linear search over the provided flags vector.
    for (search_flag_kind, search_flag_names) in &mut *flags {
      for search_flag_name in search_flag_names {
        // When searching for a flag, search_flag_name can be either in the long
        // format (--flag) or short format (-f). If the argument is a long flag,
        // it should be compared to the entire search flag name string. If it's
        // a short flag, and if the search_flag_name is also a short flag, check
        // if it ends with ch.
        if is_long && arg_flag == *search_flag_name {
          found_long = true;
        } else if !is_long &&
                  search_flag_name.len() == 2 &&
                  search_flag_name.ends_with(ch)
        {
          found_short = true;
        } else {
          // We didn't find anything.
          continue;
        }

        // Flags that take a value cannot be combined.
        if let Some(first) = last_short_flag_with_value {
          let error = FlagError { error_type: FlagErrorType::CannotCombine,
                                  flag: format!("-{}", first) };
          return Err(error);
        }

        match search_flag_kind {
          FlagType::BoolFlag(value) => {
            // A value with a boolean flag? Phew.
            if arg_val.is_some() {
              let flag_name = if !is_long {
                format!("-{}", ch)
              } else {
                arg_flag.to_string()
              };
              let error = FlagError { error_type:
                                        FlagErrorType::ExtraValueProvided,
                                      flag: flag_name };
              return Err(error);
            }
            **value = true;
            break;
          }

          FlagType::StringFlag(value) => {
            if let Some(v) = arg_val {
              **value = v.to_string();
            } else if let Some(next_arg) = args.next() {
              value.clone_from(&next_arg);
            } else {
              let error = FlagError { error_type:
                                        FlagErrorType::NoValueProvided,
                                      flag: arg_flag.to_string() };
              return Err(error);
            }
            if !is_long {
              last_short_flag_with_value = Some(ch);
            }
          }

          FlagType::RepeatFlag(value) => {
            **value += 1;
          }

          FlagType::ManyFlag(vec) => {
            if let Some(v) = arg_val {
              vec.push(v.to_string());
            } else if let Some(next_arg) = args.next() {
              vec.push(next_arg.clone());
            } else {
              let error = FlagError { error_type:
                                        FlagErrorType::NoValueProvided,
                                      flag: arg_flag.to_string() };
              return Err(error);
            }
            if !is_long {
              last_short_flag_with_value = Some(ch);
            }
          }
        }
      }
    }

    if found_long {
      break;
    } else if is_long {
      let error = FlagError { error_type: FlagErrorType::Unknown,
                              flag: arg_flag.to_string() };
      return Err(error);
    }

    // We saw every character and haven't matched anything.
    if !found_short {
      let error = FlagError { error_type: FlagErrorType::Unknown,
                              flag: format!("-{}", ch) };
      return Err(error);
    }
  }

  Ok(true)
}

/// Consumes and parses flags and arguments from
/// [`Iterator<String>`](type@Iterator<String>).
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
pub fn parse_flags<Args>(args: &mut Args,
                         flags: &mut [Flag])
                         -> Result<Vec<String>, FlagError>
  where Args: Iterator<Item = String>
{
  #[cfg(debug_assertions)]
  check_flags(flags);

  let mut parsed_arguments: Vec<String> = vec![];
  let mut ignore_rest = false;

  while let Some(arg) = args.next() {
    // Treat the rest of the input as arguments after encountering '--'.
    if arg == "--" {
      ignore_rest = true;
      continue;
    }

    // Treat '-' as an argument, otherwise try to parse a flag.
    if ignore_rest || arg == "-" || !parse_arg(&arg, args, flags)? {
      parsed_arguments.push(arg);
    }
  }

  Ok(parsed_arguments)
}

/// Works the same way as [`parse_flags`](fn@parse_flags), but stops when it
/// encounters the first argument. Consumes all flags before the first argument
/// from `args` iterator, so `args` can be used again to parse the remaining
/// contents. Will return `Ok("".to_string())` if no arguments were provided.
///
/// # Returns
/// ## Ok
/// First argument that is not a flag, or empty string when there is no
/// arguments. Changes references passed in the enums according to parsed flags.
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
pub fn parse_flags_until_subcommand<Args>(args: &mut Args,
                                          flags: &mut [Flag])
                                          -> Result<String, FlagError>
  where Args: Iterator<Item = String>
{
  #[cfg(debug_assertions)]
  check_flags(flags);

  while let Some(arg) = args.next() {
    // Treat '-'/'--' as arguments, otherwise try to parse a flag.
    if arg == "-" || arg == "--" || !parse_arg(&arg, args, flags)? {
      return Ok(arg);
    }
  }

  Ok("".to_string())
}

#[cfg(test)]
mod tests
{
  use super::*;

  #[test]
  fn flags_macro()
  {
    let mut color = String::new();
    let mut show_help = false;

    let flags = vec![(FlagType::StringFlag(&mut color), vec!["--color", "-c"]),
                     (FlagType::BoolFlag(&mut show_help), vec!["--help"]),];

    let mut color_macro;
    let mut show_help_macro;

    let flags_macro = flags!(
        color_macro: StringFlag,   ["--color", "-c"],
        show_help_macro: BoolFlag, ["--help"]
    );

    assert_eq!(flags, flags_macro);
  }

  #[test]
  fn flag_everything_after()
  {
    let argv = vec!["program", "-v", "-rr", "--", "argument", "-file",
                    "hello!", "-rrrr"];
    let mut args = argv.iter().map(|x| x.to_string());

    let mut v;
    let mut r;

    let mut flags = flags![
        v: RepeatFlag, ["-v"],
        r: RepeatFlag, ["-r"]
    ];

    let parsed_args = parse_flags(&mut args, &mut flags);

    assert_eq!(parsed_args.unwrap(),
               vec!["program", "argument", "-file", "hello!", "-rrrr"]);
    assert_eq!(v, 1);
    assert_eq!(r, 2);
  }

  #[test]
  fn flag_everything_after_subcommand()
  {
    let argv = vec!["-v", "-rr", "--", "argument"];
    let mut args = argv.iter().map(|x| x.to_string());

    let mut v;
    let mut r;

    let mut flags = flags![
        v: RepeatFlag, ["-v"],
        r: RepeatFlag, ["-r"]
    ];

    let subcommand = parse_flags_until_subcommand(&mut args, &mut flags);
    assert_eq!(&subcommand.unwrap(), "--");

    let parsed_args = parse_flags(&mut args, &mut flags);

    assert_eq!(parsed_args.unwrap(), vec!["argument"]);
    assert_eq!(v, 1);
    assert_eq!(r, 2);
  }

  #[test]
  fn flag_repeat_flag()
  {
    let argv = vec!["program", "-vvvv", "-eee", "--test", "argument"];
    let mut args = argv.iter().map(|x| x.to_string());

    let mut v;
    let mut e;
    let mut t;
    let mut unused;

    let mut flags = flags![
        v: RepeatFlag,      ["-v"],
        e: RepeatFlag,      ["-e"],
        unused: RepeatFlag, ["-u", "--unused"],
        t: RepeatFlag,      ["-t", "--test"]
    ];

    let parsed_args = parse_flags(&mut args, &mut flags);

    assert_eq!(v, 4);
    assert_eq!(e, 3);
    assert_eq!(unused, 0);
    assert_eq!(t, 1);
    assert_eq!(parsed_args.unwrap(), vec!["program", "argument"]);
  }
  #[test]
  fn parse_flags_equals()
  {
    let argv = vec!["program",
                    "arg_one",
                    "-s=test1",
                    "arg_two",
                    "--long=test2",
                    "--many=first",
                    "arg_three",
                    "--many",
                    "second",
                    "arg_four",];
    let mut args = argv.iter().map(|x| x.to_string());

    let mut s;
    let mut s2;
    let mut m;

    let mut flags = flags![
        s: StringFlag,  ["-s"],
        s2: StringFlag, ["--long"],
        m: ManyFlag,    ["--many"]
    ];

    let parsed_args = parse_flags(&mut args, &mut flags).unwrap();

    assert_eq!(parsed_args,
               vec!["program", "arg_one", "arg_two", "arg_three", "arg_four"]);
    assert_eq!(s, "test1");
    assert_eq!(s2, "test2");
    assert_eq!(m, vec!["first", "second"]);
  }

  #[test]
  fn parse_flags_default()
  {
    let argv = vec!["program",
                    "argument_one",
                    "-aVns",
                    "--long-specific",
                    "something",
                    "-vvvvv",
                    "--many",
                    "first",
                    "--many",
                    "second",
                    "argument_two",];
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

    let mut flags =
      vec![(FlagType::BoolFlag(&mut a), vec!["-a"]),
           (FlagType::BoolFlag(&mut big_v), vec!["-V"]),
           (FlagType::BoolFlag(&mut n), vec!["-n"]),
           (FlagType::BoolFlag(&mut s), vec!["-s"]),
           (FlagType::StringFlag(&mut long_specific), vec!["--long-specific"]),
           (FlagType::StringFlag(&mut not_used), vec!["--not-used"]),
           (FlagType::RepeatFlag(&mut v), vec!["-v"]),
           (FlagType::BoolFlag(&mut z), vec!["-z"]),
           (FlagType::ManyFlag(&mut many), vec!["--many"]),];

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
  fn parse_flags_no_arguments()
  {
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
  fn parse_flags_subcommand()
  {
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
  fn parse_flags_subcommand_no_argument()
  {
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
  fn parse_flags_malformed()
  {
    let args_vector = vec!["program".to_string()];

    let mut malformed = false;

    let mut flags = vec![(FlagType::BoolFlag(&mut malformed), vec!["m"])];

    parse_flags(&mut args_vector.into_iter(), &mut flags).unwrap();
  }

  #[test]
  #[should_panic]
  #[cfg(debug_assertions)]
  fn parse_flags_malformed_long()
  {
    let args_vector = vec!["program".to_string()];

    let mut malformed = false;

    let mut flags =
      vec![(FlagType::BoolFlag(&mut malformed), vec!["-onedash"])];

    parse_flags(&mut args_vector.into_iter(), &mut flags).unwrap();
  }

  #[test]
  #[should_panic]
  #[cfg(debug_assertions)]
  fn parse_flags_malformed_space()
  {
    let args_vector = vec!["program".to_string()];

    let mut malformed = false;

    let mut flags =
      vec![(FlagType::BoolFlag(&mut malformed), vec!["--space bar"])];

    parse_flags(&mut args_vector.into_iter(), &mut flags).unwrap();
  }

  #[test]
  #[should_panic]
  #[cfg(debug_assertions)]
  fn parse_flags_cant_combine()
  {
    let args_vector =
      vec!["program".to_string(), "-sa".to_string(), "test".to_string()];

    let mut s = String::new();
    let mut a = false;

    let mut flags = vec![(FlagType::StringFlag(&mut s), vec!["-s"]),
                         (FlagType::BoolFlag(&mut a), vec!["-a"])];

    parse_flags(&mut args_vector.into_iter(), &mut flags).unwrap();
  }

  #[test]
  #[should_panic]
  #[cfg(debug_assertions)]
  fn parse_flags_extra_value()
  {
    let args_vector = vec!["program".to_string(), "-s=test".to_string()];

    let mut s = false;

    let mut flags = vec![(FlagType::BoolFlag(&mut s), vec!["-s"])];

    parse_flags(&mut args_vector.into_iter(), &mut flags).unwrap();
  }
}
