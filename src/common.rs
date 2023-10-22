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
        let path = if cfg!(windows) { "toilet\\bin\\program.exe" } else { "toilet/bin/program" };
        let name = name_from_path(&path.to_string());

        assert_eq!(name, if cfg!(windows) { "program.exe" } else { "program" });
    }
}
