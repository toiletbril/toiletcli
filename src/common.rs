//! Common functions.

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

const SUPPORTED_TERMINALS: &'static [&str] = &[
    "vte",
    "kitty",
    "mintty",
    "iterm2",
    "alacritty", // since 0.12.0
];

/// Returns `true` if current `$TERMINAL` supports underline styling.
/// Calling this first time is thread unsafe, since I don't plan on using this from separate threads.
pub fn is_underline_style_supported() -> bool {
    unsafe {
        if let Some(value) = UNDERLINE_SUPPORTED {
            value
        } else {
            if let Ok(terminal) = std::env::var("TERMINAL") {
                for supported in SUPPORTED_TERMINALS {
                    if terminal.contains(supported) {
                        UNDERLINE_SUPPORTED = Some(true);
                        return true;
                    }
                }
            }
            UNDERLINE_SUPPORTED = Some(false);
            return false;
        }
    }
}

static mut USE_COLORS: Option<bool> = None;

/// Returns `false` when:
/// `$NO_COLOR` it set to anything
/// `$TERM` = `dumb`
/// `stderr` or `stdout` is not a tty
pub fn should_use_colors() -> bool {
    unsafe {
        if let Some(value) = USE_COLORS {
            value
        } else {
            let is_atty = is(Stream::Stderr) && is(Stream::Stdout);
            let use_color = !(std::env::var("NO_COLOR").is_ok() || std::env::var("TERM") == Ok("dumb".to_string()));

            if is_atty && use_color {
                USE_COLORS = Some(true);
                true
            } else {
                USE_COLORS = Some(false);
                false
            }
        }
    }
}

/// Permanently overwrite [`should_use_colors`](fn@should_use_colors) return value.
pub unsafe fn overwrite_should_use_colors(value: bool) {
    USE_COLORS = Some(value);
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
pub fn name_from_path(path: &String) -> String {
    for (i, c) in path.chars().rev().enumerate() {
        if DIR_CHARS.contains(c) {
            return String::from(path.split_at(path.len() - i).1);
        }
    }
    return String::from(path);
}

/// Compile time assertion.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn program_name() {
        let path = if cfg!(windows) {
            "toilet\\bin\\program.exe"
        } else {
            "toilet/bin/program"
        };
        let name = name_from_path(&path.to_string());

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
