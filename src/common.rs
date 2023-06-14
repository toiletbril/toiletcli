//! Common modules and functions.

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
pub const DIR_CHARS: &'static str = if cfg!(windows) { "\\/" } else { "/" };

/// List of `TERM` enviroment variables in which underline customization is supported.
pub const UNDERLINE_SUPPORTED_TERMINALS: [&str; 4] = ["vte", "kitty", "mintty", "iterm2"];

pub fn is_underline_style_supported() -> bool {
    if let Ok(terminal) = std::env::var("TERMINAL") {
        for supported in UNDERLINE_SUPPORTED_TERMINALS {
            if terminal.contains(supported) {
                return true
            }
        }
    }
    false
}

/// Gets file name from it's path.
///
/// # Example
/// ```rust
/// use toiletcli::common;
///
/// let _name = "toilet/bin/program";
/// let name = common::name_from_path(_name);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn program_name() {
        let path = if cfg!(windows) { "toilet\\bin\\program.exe" } else { "toilet/bin/program" };
        let name = name_from_path(path);

        assert_eq!(name, if cfg!(windows) { "program.exe" } else { "program" });
    }
}
