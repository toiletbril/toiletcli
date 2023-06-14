//! Tools for ASCII terminal colors.

// TODO:
// - 256 colors
// - dyn looks scary
// - str methods

use std::{fmt::Display, str::FromStr, io::{Error, ErrorKind}};

fn esc_sq(code: String) -> String {
    if cfg!(feature = "mock_colors") {
        format!("{{color code {}}}", code)
    } else {
        format!("\u{001b}[{}m", code)
    }
}

fn last_word<'a>(string: &'a str) -> String {
    let delimiters = "-_ ";
    if let Some(value) = string.split(|ch| { delimiters.contains(ch) }).last() {
        String::from(value)
    } else {
        String::from(string)
    }
}

/// ASCII colors. `Display` writes foreground color.
/// Specific methods implement colors for background and underline.
/// Use `Style::Reset` to reset all colors and styles.
///
/// # Example
/// ```rust
/// use toiletcli::colors::Color;
/// use toiletcli::colors::AsciiColor;
///
/// println!("{}{}This is red text on blue background!",
///          Color::Red, Color::Blue.background());
/// ```
pub trait AsciiColor {
    fn fg_code(&self) -> u8;
    /// Returns foreground escape sequence for this color.
    fn foreground(&self) -> String;
    fn bg_code(&self) -> u8;
    /// Returns background escape sequence for this color.
    fn background(&self) -> String;
    /// Returns underline escape sequence for this color.
    /// Underline colors and style will only work with `vte`, `kitty`, `mintty` or `iterm2` as TERM enviroment variable.
    fn underline(&self) -> String;
}

impl Default for &dyn AsciiColor {
    fn default() -> Self {
        &Color::None
    }
}

impl std::fmt::Debug for &dyn AsciiColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// ASCII colors. `Display` writes foreground color.
/// Specific methods implement colors for background and underline.
/// Use `Style::Reset` to reset all colors and styles.
///
/// # Example
/// ```rust
/// use toiletcli::colors::Color;
/// use toiletcli::colors::AsciiColor;
///
/// println!("{}{}This is red text on blue background!",
///          Color::Red, Color::Blue.background());
/// ```
#[repr(u8)]
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Color {
    None    = 255,
    Default = 9,
    Black   = 0,
    Red     = 1,
    Green   = 2,
    Yellow  = 3,
    Blue    = 4,
    Purple  = 5,
    Cyan    = 6,
    White   = 7,
}

impl Default for Color {
    fn default() -> Self {
        Color::None
    }
}

impl AsciiColor for Color {
    #[inline(always)]
    fn fg_code(&self) -> u8 {
        30 + *self as u8
    }

    /// Returns foreground escape sequence for this color.
    fn foreground(&self) -> String {
        if *self != Color::None {
            esc_sq(self.fg_code().to_string())
        }
        else {
            String::new()
        }
    }

    #[inline(always)]
    fn bg_code(&self) -> u8 {
        40 + *self as u8
    }

    /// Returns background escape sequence for this color.
    fn background(&self) -> String {
        if *self != Color::None {
            esc_sq(self.bg_code().to_string())
        }
        else {
            String::new()
        }
    }

    /// Returns underline escape sequence for this color.
    /// Underline colors and style will only work with `vte`, `kitty`, `mintty` or `iterm2` as TERM enviroment variable.
    fn underline(&self) -> String {
        if *self != Color::None {
            esc_sq(format!("58;5;{}", self.fg_code()))
        } else {
            String::new()
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.foreground())
    }
}

impl FromStr for Color {
    type Err = Error;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let color = match last_word(string).to_lowercase().as_str() {
            "default" => Color::Default,
            "black"   => Color::Black,
            "red"     => Color::Red,
            "green"   => Color::Green,
            "yellow"  => Color::Yellow,
            "blue"    => Color::Blue,
            "purple"  => Color::Purple,
            "cyan"    => Color::Cyan,
            "white"   => Color::White,
            _ => {
                return Err(Error::new(ErrorKind::Other, format!("Unknown color '{}'.", string)))
            }
        };

        Ok(color)
    }
}


/// ASCII colors. `Display` writes foreground color.
/// Specific methods implement colors for background and underline.
/// Use `Style::Reset` to reset all colors and styles.
///
/// # Example
/// ```rust
/// use toiletcli::colors::Color;
/// use toiletcli::colors::AsciiColor;
///
/// println!("{}{}This is red text on blue background!",
///          Color::Red, Color::Blue.background());
/// ```
#[repr(u8)]
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum BrightColor {
    None    = 255,
    Default = 9,
    Black   = 0,
    Red     = 1,
    Green   = 2,
    Yellow  = 3,
    Blue    = 4,
    Purple  = 5,
    Cyan    = 6,
    White   = 7,
}

impl Default for BrightColor {
    fn default() -> Self {
        BrightColor::None
    }
}

impl AsciiColor for BrightColor {
    #[inline(always)]
    fn fg_code(&self) -> u8 {
        90 + *self as u8
    }

    /// Returns foreground escape sequence for this color.
    fn foreground(&self) -> String {
        if *self != BrightColor::None {
            esc_sq(self.fg_code().to_string())
        }
        else {
            String::new()
        }
    }

    #[inline(always)]
    fn bg_code(&self) -> u8 {
        100 + *self as u8
    }

    /// Returns background escape sequence for this color.
    fn background(&self) -> String {
        if *self != BrightColor::None {
            esc_sq(self.bg_code().to_string())
        }
        else {
            String::new()
        }
    }

    /// Returns underline escape sequence for this color.
    /// Underline colors and style will only work with `vte`, `kitty`, `mintty` or `iterm2` as TERM enviroment variable.
    fn underline(&self) -> String {
        if *self != BrightColor::None {
            esc_sq(format!("58;5;{}", self.fg_code()))
        } else {
            String::new()
        }
    }
}

impl Display for BrightColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.foreground())
    }
}

impl FromStr for BrightColor {
    type Err = Error;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if !string.starts_with("bright") {
            return Err(Error::new(ErrorKind::Other, format!("Unknown color '{}'.", string)))
        }
        let color = match last_word(string).to_lowercase().as_str() {
            "default" => BrightColor::Default,
            "black"   => BrightColor::Black,
            "red"     => BrightColor::Red,
            "green"   => BrightColor::Green,
            "yellow"  => BrightColor::Yellow,
            "blue"    => BrightColor::Blue,
            "purple"  => BrightColor::Purple,
            "cyan"    => BrightColor::Cyan,
            "white"   => BrightColor::White,
            _ => {
                return Err(Error::new(ErrorKind::Other, format!("Unknown color '{}'.", string)))
            }
        };

        Ok(color)
    }
}

/// ASCII terminal style codes. Can be used via `Display`.
///
/// # Example
/// ```rust
/// use toiletcli::colors::Style;
///
/// println!("{}{}This is bold italic text!",
///          Style::Bold, Style::Italic);
/// ```
#[repr(u8)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Style {
    None          = 22,
    Reset         = 0,
    Bold          = 1,
    Faint         = 2,
    Italic        = 3,
    Underlined    = 4,
    Strikethrough = 9,
}

impl Default for Style {
    fn default() -> Self {
        Style::None
    }
}

impl Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if *self == Style::None {
            return Ok(());
        }
        write!(f, "{}", esc_sq((*self as u8).to_string()))
    }
}

impl FromStr for Style {
    type Err = Error;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string.to_lowercase().as_str() {
            "none"          |
            ""              => Ok(Style::None),
            "reset"         => Ok(Style::Reset),
            "bold"          => Ok(Style::Bold),
            "faint"         => Ok(Style::Faint),
            "italic"        => Ok(Style::Italic),
            "underlined"    |
            "underline"     => Ok(Style::Underlined),
            "strikethrough" |
            "striked"       |
            "crossed"       => Ok(Style::Strikethrough),
            _ => {
                Err(Error::new(ErrorKind::Other, format!("Unknown style '{}'.", string)))
            }
        }
    }
}

const UNDERLINE_SUPPORTED_TERMINALS: [&str; 4] = ["vte", "kitty", "mintty", "iterm2"];

fn underline_supported() -> bool {
    if let Ok(terminal) = std::env::var("TERMINAL") {
        for supported in UNDERLINE_SUPPORTED_TERMINALS {
            if terminal.contains(supported) {
                return true
            }
        }
    }
    false
}

/// Underline styles.
/// Underline colors and style will only work with `vte`, `kitty`, `mintty` or `iterm2` as TERM enviroment variable.
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UnderlineStyle {
    None          = 24,
    Default       = 0,
    Straight      = 1,
    Double        = 2,
    Curly         = 3,
    Dotted        = 4,
    Dashed        = 5,
}

impl Default for UnderlineStyle {
    fn default() -> Self {
        UnderlineStyle::None
    }
}

impl Display for UnderlineStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if *self == UnderlineStyle::None {
            return write!(f, "{}", esc_sq("24".to_string()));
        }
        write!(f, "{}", esc_sq(format!("4:{}", (*self as u8))))
    }
}

/// Structure with builder methods to save a specific style and colors.
/// Underline colors and style will only work with `vte`, `kitty`, `mintty` or `iterm2` as TERM enviroment variable.
///
/// # Example
/// ```rust
/// use toiletcli::colors::*;
///
/// let mut my_style = TerminalStyle::new();
/// my_style
///     .set_bg(&BrightColor::Black)
///     .add_style(Style::Italic)
///     .add_style(Style::Underlined);
///
/// println!("{}This is underlined and italic text on bright black background!", my_style);
/// ```
#[derive(Default, Debug)]
pub struct TerminalStyle<'a> {
    foreground: &'a dyn AsciiColor,
    background: &'a dyn AsciiColor,
    styles: Vec<Style>,
    underline_color: &'a dyn AsciiColor,
    underline_style: UnderlineStyle,
}

impl<'a> Display for TerminalStyle<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Style::Reset)?;

        write!(f, "{}", self.foreground.foreground())?;
        write!(f, "{}", self.background.background())?;

        for style in &self.styles {
            write!(f, "{}", style)?;
        }

        if underline_supported() {
            let _ = write!(f, "{}", self.underline_color.underline())?;
            let _ = write!(f, "{}", self.underline_style)?;
        }

        Ok(())
    }
}

impl<'a> TerminalStyle<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_fg(&mut self, color: &'a dyn AsciiColor) -> &mut Self {
        self.foreground = color;
        self
    }

    pub fn set_bg(&mut self, color: &'a dyn AsciiColor) -> &mut Self {
        self.background = color;
        self
    }

    /// Add a style to text. Can be used multiple times.
    pub fn add_style(&mut self, style: Style) -> &mut Self {
        self.styles.push(style);
        self
    }

    /// Underline colors and style will only work with `vte`, `kitty`, `mintty` or `iterm2` as TERM enviroment variable.
    pub fn set_underline_style(&mut self, underline_style: UnderlineStyle) -> &mut Self {
        self.underline_style = underline_style;
        self
    }

    /// Underline colors and style will only work with `vte`, `kitty`, `mintty` or `iterm2` as TERM enviroment variable.
    pub fn set_underline_color(&mut self, underline_color: &'a dyn AsciiColor) -> &mut Self {
        self.underline_color = underline_color;
        self
    }

    /// Clear all styles and colors.
    pub fn clear(&mut self) -> &mut Self {
        *self = TerminalStyle::new();
        self
    }
}

/// Run with `--nocapture`
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn red() {
        println!("{}Red text!{}",
                 Color::Red, Style::Reset);
    }

    #[test]
    fn blue_on_red() {
        println!("{}{}Blue on bright red text!{}",
                 Color::Blue, BrightColor::Red.background(), Style::Reset);
    }

    #[test]
    fn italic_bold_cyan() {
        println!("{}{}{}Italic bold cyan text!{}",
                 Style::Italic, Style::Bold, Color::Cyan, Style::Reset);
    }

    #[test]
    fn terminal_style() {
        let mut my_style = TerminalStyle::new();
        my_style
            .set_bg(&BrightColor::Black)
            .add_style(Style::Italic)
            .add_style(Style::Strikethrough);

        println!("{}This is striked and italic text on bright black background!{}", my_style, Style::Reset);
    }

    #[test]
    fn many_styles() {
        let mut warning = TerminalStyle::new();
        warning
            .set_fg(&BrightColor::Black)
            .set_bg(&Color::Red)
            .add_style(Style::Bold);

        let mut warning_message = TerminalStyle::new();
        warning_message
            .set_fg(&Color::Purple)
            .add_style(Style::Underlined)
            .set_underline_style(UnderlineStyle::Curly);

        println!("{}Bold bright black on red:{} {}Underlined curly purple!{}",
                 warning, Style::Reset, warning_message, Style::Reset);
    }
}
