//! Tools for ASCII terminal colors.
//! Contains enums that all implement `Display` and `FromStr` traits.

use crate::common::is_underline_style_supported;

use std::{fmt::Display, str::FromStr, io::{Error, ErrorKind}};

#[inline(always)]
fn esc_sq(code: String) -> String {
    if code.len() != 0 {
        if cfg!(feature = "mock_colors") {
            format!("{{color code {}}}", code)
        } else {
            format!("\u{001b}[{}m", code)
        }
    } else {
        String::new()
    }
}

/// Methods which return printable colors.
/// Use `Style::Reset` to reset all colors and styles.
///
/// # Example
/// ```rust
/// use toiletcli::colors::Color;
/// use toiletcli::colors::PrintableColor;
///
/// println!("{}{}This is red text on blue background!",
///          Color::Red, Color::Blue.background());
/// ```
pub trait PrintableColor {
    /// Returns foreground escape sequence for this color.
    fn foreground(&self) -> String;
    /// Returns background escape sequence for this color.
    fn background(&self) -> String;
    /// Returns underline escape sequence for this color.
    /// Underline colors and style will only work with `vte`, `kitty`, `mintty` or `iterm2` as TERM enviroment variable.
    fn underline(&self) -> String;
    /// Converts this color to 8-bit color.
    fn byte(&self) -> u8;
}

/// ASCII colors. `Display` writes foreground color.
/// Includes `Color::RGB(u8, u8, u8)` and `Color::Byte(u8)` which reprent RGB colors and 8-bit colors.
/// This implements `PrintableColor` methods, which also return colors for background and underline.
/// Use `Style::Reset` to reset all colors and styles.
///
/// # Example
/// ```rust
/// use toiletcli::colors::Color;
/// use toiletcli::colors::PrintableColor;
///
/// println!("{}{}This is red text on blue background!",
///          Color::Red, Color::Blue.background());
/// ```
#[repr(u8)]
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Color {
    None         = 255,
    Black        = 0,
    Red          = 1,
    Green        = 2,
    Yellow       = 3,
    Blue         = 4,
    Purple       = 5,
    Cyan         = 6,
    White        = 7,
    BrightBlack  = 8,
    BrightRed    = 9,
    BrightGreen  = 10,
    BrightYellow = 11,
    BrightBlue   = 12,
    BrightPurple = 13,
    BrightCyan   = 14,
    BrightWhite  = 15,
    /// 8-bit color.
    Byte(u8),
    /// RGB color.
    RGB(u8, u8, u8)
}

impl Default for Color {
    fn default() -> Self {
        Color::None
    }
}

/// Convert RGB color to closest 8-bit color.
#[inline(always)]
pub fn rgb_to_byte(r: u8, g: u8, b: u8) -> u8 {
    (16 + ((r as u32 * 6 / 256) * 36) + ((g as u32 * 6 / 256) * 6) + (b as u32 * 6 / 256)) as u8
}

impl From<Color> for u8 {
    fn from(value: Color) -> Self {
        match value {
            Color::None         => panic!("Cant convert Color::None to u8"),
            Color::Black        => 0,
            Color::Red          => 1,
            Color::Green        => 2,
            Color::Yellow       => 3,
            Color::Blue         => 4,
            Color::Purple       => 5,
            Color::Cyan         => 6,
            Color::White        => 7,
            Color::BrightBlack  => 8,
            Color::BrightRed    => 9,
            Color::BrightGreen  => 10,
            Color::BrightYellow => 11,
            Color::BrightBlue   => 12,
            Color::BrightPurple => 13,
            Color::BrightCyan   => 14,
            Color::BrightWhite  => 15,
            Color::Byte(color)  => color,
            Color::RGB(r, g, b) => rgb_to_byte(r, g, b)
        }
    }
}

impl From<u8> for Color {
    fn from(value: u8) -> Self {
        match value {
            0  => Color::Black,
            1  => Color::Red,
            2  => Color::Green,
            3  => Color::Yellow,
            4  => Color::Blue,
            5  => Color::Purple,
            6  => Color::Cyan,
            7  => Color::White,
            8  => Color::BrightBlack,
            9  => Color::BrightRed,
            10 => Color::BrightGreen,
            11 => Color::BrightYellow,
            12 => Color::BrightBlue,
            13 => Color::BrightPurple,
            14 => Color::BrightCyan,
            15 => Color::BrightWhite,
            color => Color::Byte(color),
        }
    }
}

impl Color {
    fn fg_code(&self) -> String {
        match self {
            Color::None         => "".to_string(),
            Color::Byte(color)  => format!("38;5;{}", color),
            Color::RGB(r, g, b) => format!("38;2;{};{};{}", r, g, b),
            _ => {
                let color = u8::from(*self);

                if color < 8 {
                    format!("3{}", color)
                } else {
                    format!("9{}", color - 8)
                }
            }
        }
    }

    fn bg_code(&self) -> String {
        match self {
            Color::None         => "".to_string(),
            Color::Byte(color)  => format!("48;5;{}", color),
            Color::RGB(r, g, b) => format!("48;2;{};{};{}", r, g, b),
            _ => {
                let color = u8::from(*self);

                if color < 8 {
                    format!("4{}", color)
                } else {
                    format!("10{}", color - 8)
                }
            }
        }
    }

    fn ul_code(&self) -> String {
        match self {
            Color::None         => "".to_string(),
            Color::Byte(color)  => format!("58;5;{}", color),
            Color::RGB(r, g, b) => format!("58;2;{};{};{}", r, g, b),
            _                   => format!("58;5;{}", self.byte()),
        }
    }
}

impl PrintableColor for Color {
    /// Returns color byte that is closest to this color.
    fn byte(&self) -> u8 {
        match self {
            Color::Byte(byte) => *byte,
            Color::RGB(r, g, b) => {
                rgb_to_byte(*r, *g, *b)
            },
            _ => {
                let color = u8::from(*self);
                match color {
                    9       => 0,
                    0..=7   => color,
                    60..=67 => color - 52,
                    _ => unreachable!()
                }
            }
        }
    }

    /// Returns foreground escape sequence for this color.
    fn foreground(&self) -> String {
        esc_sq(self.fg_code())
    }

    /// Returns background escape sequence for this color.
    fn background(&self) -> String {
        esc_sq(self.bg_code())
    }

    /// Returns underline escape sequence for this color.
    /// Underline colors and style will only work with `vte`, `kitty`, `mintty` or `iterm2` as TERM enviroment variable.
    fn underline(&self) -> String {
        esc_sq(self.ul_code())
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
        let delimiters = "-_ ";

        for (index, _) in string.match_indices(|ch| { delimiters.contains(ch) }) {
            println!("{}", string);
            if string[..index] == *"bright" {
                return Ok(
                    match &string[index + 1..] {
                        "none"    => Color::None,
                        "black"   => Color::BrightBlack,
                        "red"     => Color::BrightRed,
                        "green"   => Color::BrightGreen,
                        "yellow"  => Color::BrightYellow,
                        "blue"    => Color::BrightBlue,
                        "purple"  => Color::BrightPurple,
                        "cyan"    => Color::BrightCyan,
                        "white"   => Color::BrightWhite,
                        _ => {
                            return Err(Error::new(ErrorKind::Other, format!("Unknown color '{}'.", string)))
                        }
                    }
                )
            }
        }
        Ok(match string {
                "none"    => Color::None,
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
            }
        )
    }
}

/// Terminal style codes. Can be used via `Display`.
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
    Default       = 22,
    Reset         = 0,
    Bold          = 1,
    Faint         = 2,
    Italic        = 3,
    Underlined    = 4,
    Strikethrough = 9,
}

impl Default for Style {
    fn default() -> Self {
        Style::Default
    }
}

impl Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", esc_sq((*self as u8).to_string()))
    }
}

impl FromStr for Style {
    type Err = Error;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string.to_lowercase().as_str() {
            "default"       => Ok(Style::Default),
            "none"          |
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

/// Underline styles.
/// Underline colors and style will only work with `vte`, `kitty`, `mintty` or `iterm2` as TERM enviroment variable.
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UnderlineStyle {
    None          = 0,
    Straight      = 1,
    Double        = 2,
    Curly         = 3,
    Dotted        = 4,
    Dashed        = 5,
}

impl Default for UnderlineStyle {
    fn default() -> Self {
        UnderlineStyle::Straight
    }
}

impl UnderlineStyle {
    fn code(&self) -> String {
        format!("4:{}", *self as u8)
    }
}

impl Display for UnderlineStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if *self == UnderlineStyle::None {
            return write!(f, "{}", esc_sq("24".to_string()));
        }
        write!(f, "{}", esc_sq(self.code()))
    }
}

impl FromStr for UnderlineStyle {
    type Err = Error;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string.to_lowercase().as_str() {
            "default"       |
            "straight"      => Ok(UnderlineStyle::Straight),
            "reset"         |
            "none"          => Ok(UnderlineStyle::None),
            "double"        => Ok(UnderlineStyle::Double),
            "curly"         => Ok(UnderlineStyle::Curly),
            "dotted"        => Ok(UnderlineStyle::Dotted),
            "dashed"        => Ok(UnderlineStyle::Dashed),
            _ => {
                Err(Error::new(ErrorKind::Other, format!("Unknown underline style '{}'.", string)))
            }
        }
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
///     .background(Color::BrightBlack)
///     .add_style(Style::Italic)
///     .add_style(Style::Underlined);
///
/// println!("{}This is underlined and italic text on bright black background!", my_style);
/// ```
#[derive(Default, Debug)]
pub struct TerminalStyle {
    foreground: Color,
    background: Color,
    styles: Vec<Style>,
    underline_color: Color,
    underline_style: UnderlineStyle,
}

fn is_code_closed(string: &String) -> bool {
    if let Some(last) = string.chars().last() {
        last == ';'
    } else {
        true
    }
}

#[inline(always)]
fn close_and_concat(code_string: &mut String, style: String) {
    if is_code_closed(code_string) {
        *code_string = format!("{}{}", code_string, style)
    } else {
        *code_string = format!("{};{}", code_string, style)
    }
}

impl Display for TerminalStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut code_string = String::new();

        close_and_concat(&mut code_string, self.foreground.fg_code());
        close_and_concat(&mut code_string, self.background.bg_code());

        for style in &self.styles {
            close_and_concat(&mut code_string, (*style as u8).to_string());
        }

        if is_underline_style_supported() {
            close_and_concat(&mut code_string, self.underline_color.ul_code());
            close_and_concat(&mut code_string, self.underline_style.code());
        }

        let _ = write!(f, "{}", esc_sq(code_string))?;

        Ok(())
    }
}

impl TerminalStyle {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn foreground(&mut self, color: Color) -> &mut Self {
        self.foreground = color;
        self
    }

    pub fn background(&mut self, color: Color) -> &mut Self {
        self.background = color;
        self
    }

    /// Add a style to text. Can be used multiple times.
    pub fn add_style(&mut self, style: Style) -> &mut Self {
        self.styles.push(style);
        self
    }

    /// Underline colors and style will only work with `vte`, `kitty`, `mintty` or `iterm2` as TERM enviroment variable.
    pub fn underline_style(&mut self, underline_style: UnderlineStyle) -> &mut Self {
        self.underline_style = underline_style;
        self
    }

    /// Underline colors and style will only work with `vte`, `kitty`, `mintty` or `iterm2` as TERM enviroment variable.
    pub fn underline_color(&mut self, underline_color: Color) -> &mut Self {
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
    fn rgb_conversion() {
        assert_eq!(Color::RGB(255, 0, 0).byte(), 196);
        assert_eq!(Color::RGB(0, 255, 0).byte(), 46);
        assert_eq!(Color::RGB(0, 0, 255).byte(), 21);
    }
}
