//! ANSI terminal colors.

use std::{fmt::Display, str::FromStr, io::{Error, ErrorKind}};

use crate::common::is_underline_style_supported;

#[inline(always)]
fn esc_sq(code: String) -> String {
    if code.len() != 0 {
        if cfg!(feature = "mock_codes") {
            format!("{{code {}}}", code)
        } else {
            format!("\u{001b}[{}m", code)
        }
    } else {
        String::new()
    }
}

/// ANSI, RGB, 8-bit colors. `Display` writes foreground color.
///
/// Can be parsed from string with `from_str` or `&str.parse::<Color>()`. For example, `"21"` will be parsed as 8-bit color, and `"bright red"` (`'-'` or `'_'` can be used instead of space) will be parsed as standard colors.
///
/// # Example
/// ```rust
/// use toiletcli::colors::Color;
///
/// println!("{}{}This is red text on blue background!",
///          Color::Red, Color::Blue.bg());
/// ```
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Color {
    // Does nothing.
    None         = 255,
    Reset        = 254,
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

#[inline(always)]
fn rgb_to_byte(r: u8, g: u8, b: u8) -> u8 {
    (16 + ((r as u32 * 6 / 256) * 36) + ((g as u32 * 6 / 256) * 6) + (b as u32 * 6 / 256)) as u8
}

impl From<Color> for u8 {
    fn from(value: Color) -> Self {
        match value {
            Color::None         |
            Color::Reset        => panic!("Can't convert {:?} to u8", value),
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
            Color::Reset        => "39".to_string(),
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
            Color::Reset        => "49".to_string(),
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
            Color::Reset        => "59".to_string(),
            Color::Byte(color)  => format!("58;5;{}", color),
            Color::RGB(r, g, b) => format!("58;2;{};{};{}", r, g, b),
            _                   => format!("58;5;{}", self.byte()),
        }
    }
}

impl Color {
    /// Returns color byte that is closest to this color.
    pub fn byte(&self) -> u8 {
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
    pub fn fg(&self) -> String {
        esc_sq(self.fg_code())
    }

    /// Returns background escape sequence for this color.
    pub fn bg(&self) -> String {
        esc_sq(self.bg_code())
    }

    /// Returns underline escape sequence for this color.
    pub fn ul(&self) -> String {
        esc_sq(self.ul_code())
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.fg())
    }
}

impl FromStr for Color {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let delimiters = "-_ ";

        for (index, _) in string.match_indices(|ch| { delimiters.contains(ch) }) {
            if string[..index] == *"bright" {
                return Ok(
                    match &string[index + 1..] {
                        "none"   => Color::None,
                        "black"  => Color::BrightBlack,
                        "red"    => Color::BrightRed,
                        "green"  => Color::BrightGreen,
                        "yellow" => Color::BrightYellow,
                        "blue"   => Color::BrightBlue,
                        "purple" => Color::BrightPurple,
                        "cyan"   => Color::BrightCyan,
                        "white"  => Color::BrightWhite,
                        _ => {
                            return Err(Error::new(ErrorKind::Other, format!("Unknown color '{}'.", string)))
                        }
                    }
                )
            }
        }
        Ok(match string {
                "none"   => Color::None,
                "black"  => Color::Black,
                "red"    => Color::Red,
                "green"  => Color::Green,
                "yellow" => Color::Yellow,
                "blue"   => Color::Blue,
                "purple" => Color::Purple,
                "cyan"   => Color::Cyan,
                "white"  => Color::White,
                "gray"   => Color::BrightBlack,
                _ => {
                    if let Ok(byte) = string.parse::<u8>() {
                        Color::Byte(byte)
                    } else {
                        return Err(Error::new(ErrorKind::Other, format!("Unknown color '{}'.", string)))
                    }
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
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Style {
    // Does nothing.
    None               = 255,
    // Resets all colors and styles.
    Reset              = 0,
    Bold               = 1,
    Faint              = 2,
    Italic             = 3,
    Underlined         = 4,
    Strikethrough      = 9,
    ResetBold          = 22,
    ResetItalic        = 23,
    ResetUnderline     = 24,
    ResetStrikethrough = 29,
}

impl Default for Style {
    fn default() -> Self {
        Style::None
    }
}

impl Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if *self != Style::None {
            write!(f, "{}", esc_sq((*self as u8).to_string()))
        } else {
            Ok(())
        }
    }
}

impl FromStr for Style {
    type Err = Error;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string.to_lowercase().as_str() {
            "default"       |
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

/// Underline style codes. Will not be used on incompatible terminals. Can be used via `Display`.
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UnderlineStyle {
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
        write!(f, "{}", esc_sq(self.code()))
    }
}

impl FromStr for UnderlineStyle {
    type Err = Error;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string.to_lowercase().as_str() {
            "default"       |
            "straight"      => Ok(UnderlineStyle::Straight),
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

/// Structure to save a specific style and colors.
/// Use `StyleBuilder` to construct this.
#[derive(Default, Debug, Clone)]
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
fn concat_codes(code_string: &mut String, style: &String) {
    if style.is_empty() {
        ()
    }
    else if is_code_closed(code_string) {
        *code_string = format!("{}{}", code_string, style)
    } else {
        *code_string = format!("{};{}", code_string, style)
    }
}

impl Display for TerminalStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut code_string = String::new();

        concat_codes(&mut code_string, &self.foreground.fg_code());
        concat_codes(&mut code_string, &self.background.bg_code());

        for style in &self.styles {
            concat_codes(&mut code_string, &(*style as u8).to_string());
        }

        if is_underline_style_supported() {
            concat_codes(&mut code_string, &self.underline_color.ul_code());
            if self.underline_color != Color::None {
                concat_codes(&mut code_string, &self.underline_style.code());
            }
        }

        let _ = write!(f, "{}", esc_sq(code_string))?;

        Ok(())
    }
}

/// Builder for `TerminalStyle`.
///
/// # Example
/// ```rust
/// use toiletcli::colors::*;
///
/// let weird_style = StyleBuilder::new()
///     .foreground(Color::Byte(93))
///     .add_style(Style::Underlined)
///     .underline_color(Color::RGB(0, 255, 0))
///     .underline_style(UnderlineStyle::Curly)
///     .build();
///
/// println!("{}RGB purple with RGB curly green underline!{}",
///          weird_style, Style::Reset);
/// ```
pub struct StyleBuilder {
    style: TerminalStyle
}

impl StyleBuilder {
    pub fn new() -> Self {
        StyleBuilder { style: Default::default() }
    }

    pub fn foreground(&mut self, color: Color) -> &mut Self {
        self.style.foreground = color;
        self
    }

    pub fn background(&mut self, color: Color) -> &mut Self {
        self.style.background = color;
        self
    }

    /// Add a style to text. Can be used multiple times.
    pub fn add_style(&mut self, style: Style) -> &mut Self {
        self.style.styles.push(style);
        self
    }

    /// Underline colors and style will only work on supported terminals.
    pub fn underline_style(&mut self, underline_style: UnderlineStyle) -> &mut Self {
        self.style.underline_style = underline_style;
        self
    }

    /// Underline colors and style will only work on supported terminals.
    pub fn underline_color(&mut self, underline_color: Color) -> &mut Self {
        self.style.underline_color = underline_color;
        self
    }

    pub fn build(&mut self) -> TerminalStyle {
        self.style.clone()
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