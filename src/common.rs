//! Common functions.

use std::sync::Once;

extern crate atty;

use atty::is;
use atty::Stream;

/// Directory characters, which would be `"/\\"` on Windows and `"/"` on POSIX.
///
/// # Example
/// ```rust
/// use toiletcli::common::DIR_CHARS;
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
pub const DIR_CHARS: &str = if cfg!(windows) { "\\/" } else { "/" };

static mut UNDERLINE_SUPPORTED: Option<bool> = None;
static mut UNDERLINE_SUPPORTED_INIT: Once = Once::new();

const SUPPORTED_TERMINALS: &[&str] =
  &["vte", "kitty", "mintty", "iterm2", "alacritty" /* since 0.12.0 */];
const SUPPORTED_TERMS: &[&str] = &["xterm-ghostty"];

/// Returns `true` if current `$TERMINAL` supports underline styling.
pub fn is_underline_style_supported() -> bool
{
  unsafe {
    if let Some(value) = UNDERLINE_SUPPORTED {
      value
    } else {
      let is_supported = if let Ok(terminal) = std::env::var("TERMINAL") {
        SUPPORTED_TERMINALS.iter()
                           .any(|&supported| terminal.contains(supported))
      } else {
        false
      } || if let Ok(term) = std::env::var("TERM") {
        SUPPORTED_TERMS.iter().any(|&supported| term.contains(supported))
      } else {
        false
      };

      (*(&raw mut UNDERLINE_SUPPORTED_INIT)).call_once(|| {
                                              UNDERLINE_SUPPORTED =
                                                Some(is_supported);
                                            });

      is_supported
    }
  }
}

static mut USE_COLORS: Option<bool> = None;
#[allow(dead_code)]
static mut USE_COLORS_INIT: Once = Once::new();

/// If this function returns `false`, colors will be replaced with nothing.
///
/// Returns `false` when:
/// `$NO_COLOR` it set to anything
/// `$TERM` = `dumb`
/// `stderr` or `stdout` is not a tty
pub fn should_use_colors() -> bool
{
  unsafe {
    if let Some(value) = USE_COLORS {
      value
    } else {
      let is_atty = is(Stream::Stderr) && is(Stream::Stdout);
      let use_color = !(std::env::var("NO_COLOR").is_ok() ||
                        std::env::var("TERM") == Ok("dumb".to_string()));

      let should_use_colors = is_atty && use_color;

      (*(&raw mut UNDERLINE_SUPPORTED_INIT)).call_once(|| {
                                              USE_COLORS =
                                                Some(should_use_colors);
                                            });

      should_use_colors
    }
  }
}

/// Permanently overwrite [`should_use_colors`](fn@should_use_colors) return
/// value. # Safety
/// This function is not safe in multithreaded environments, since it directly
/// writes to static variable.
pub unsafe fn overwrite_should_use_colors(value: bool)
{
  USE_COLORS = Some(value);
}

#[inline(always)]
pub fn is_stdin_a_tty() -> bool
{
  is(Stream::Stdin)
}

#[inline(always)]
pub fn is_stdout_a_tty() -> bool
{
  is(Stream::Stdout)
}

#[inline(always)]
pub fn is_stderr_a_tty() -> bool
{
  is(Stream::Stderr)
}

/// Gets file name from it's path.
///
/// # Example
/// ```rust
/// use toiletcli::common;
///
/// let path = "toilet/bin/program".to_string();
/// let name = common::name_from_path(&path);
/// assert_eq!(name, "program");
/// ```
pub fn name_from_path(path: &String) -> String
{
  for (i, c) in path.chars().rev().enumerate() {
    if DIR_CHARS.contains(c) {
      return String::from(path.split_at(path.len() - i).1);
    }
  }
  String::from(path)
}

/// Compile time assertion.
#[macro_export]
macro_rules! static_assert {
  ($cond:expr) => {
    #[allow(dead_code)]
    const fn static_assertion()
    {
      assert!($cond);
    }
    const _: () = static_assertion();
  };
}

#[cfg(test)]
mod tests
{
  use super::*;

  #[test]
  fn program_name()
  {
    let path = if cfg!(windows) {
      "toilet\\bin\\program.exe"
    } else {
      "toilet/bin/program"
    };
    let name = name_from_path(&path.to_string());

    assert_eq!(name, if cfg!(windows) { "program.exe" } else { "program" });
  }
}
