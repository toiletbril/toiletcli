//! Common functions.

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

// TODO: We do a bit of races
/// Returns `true` if current terminal supports underline styling.
pub fn is_underline_style_supported() -> bool {
    unsafe {
        if let Some(value) = UNDERLINE_SUPPORTED {
            value
        } else {
            if let Ok(terminal) = std::env::var("TERMINAL") {
                for supported in ["vte", "kitty", "mintty", "iterm2"] {
                    if terminal.contains(supported) {
                        UNDERLINE_SUPPORTED = Some(true);
                        return true;
                    }
                }
                false
            } else {
                UNDERLINE_SUPPORTED = Some(false);
                false
            }
        }
    }
}

/// Gets file name from it's path.
///
/// # Example
/// ```rust
/// use toiletcli::common;
///
/// let path = "toilet/bin/program";
/// let name = common::name_from_path(path);
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
        let path = if cfg!(windows) { "toilet\\bin\\program.exe" } else { "toilet/bin/program" };
        let name = name_from_path(path);

        assert_eq!(name, if cfg!(windows) { "program.exe" } else { "program" });
    }
}
