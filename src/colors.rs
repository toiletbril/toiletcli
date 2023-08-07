//! Convenient ANSI terminal colors and styles.
//! All enums implement [`Display`](trait@Display) and [`FromStr`](trait@FromStr) traits.

use std::{fmt::Display, str::FromStr, io::{Error, ErrorKind}};

use crate::common::is_underline_style_supported;

#[inline(always)]
fn esc_sq(code: String) -> String {
    if !code.is_empty() {
        if cfg!(feature = "mock_codes") {
            format!("{{code {}}}", code)
        } else {
            format!("\u{001b}[{}m", code)
        }
    } else {
        code    
    }
}

/// ANSI, RGB, 8-bit colors. [`Display`](trait@Display) writes foreground color.
///
/// Can be parsed from string with [`from_str`](trait@FromStr) or [`&str.parse::<Color>()`](fn@str::parse<Color>). For example, `"21"` will be parsed as 8-bit color, and `"bright red"` (`'-'` or `'_'` can be used instead of spaces) will be parsed as one of 16 colors.
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
                let color_number = u8::from(*self);
                if color_number < 8 {
                    format!("3{}", color_number)
                } else {
                    format!("9{}", color_number - 8)
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
                let color_number = u8::from(*self);

                if color_number < 8 {
                    format!("4{}", color_number)
                } else {
                    format!("10{}", color_number - 8)
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
            _ => format!("58;5;{}", self.byte()),
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
        let indices = string.match_indices(|ch| { delimiters.contains(ch) });

        let mut color_string = string;
        let mut fix_value = 0;

        for (index, _) in indices {
            let raw_color = &string[index + 1..];
            let prefix = &string[..index];

            if prefix == "bright" {
                fix_value = 8;
                color_string = raw_color;
            }
        }

        let color_number = u8::from(match color_string {
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
                    let err = Error::new(
                        ErrorKind::Other, format!("Unknown color '{}'", string)
                    );
                    return Err(err)
                }
            }
        });

        let color = Color::from(color_number + fix_value);

        Ok(color)
    }
}

/// Terminal style codes.
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

impl Style {
    fn code(&self) -> String {
        format!("{}", *self as u8)
    }
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
                let err = Error::new(ErrorKind::Other, format!("Unknown style '{}'", string));
                Err(err)
            }
        }
    }
}

/// Underline style codes. Will not be used on incompatible terminals.
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UnderlineStyle {
    Straight = 1,
    Double   = 2,
    Curly    = 3,
    Dotted   = 4,
    Dashed   = 5,
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
                let err = Error::new(ErrorKind::Other, format!("Unknown underline style '{}'", string));
                Err(err)
            }
        }
    }
}

/// Structure to save a specific style and colors.
/// Use [`StyleBuilder`](struct@StyleBuilder) to construct this.
#[derive(Default, Debug, Clone)]
pub struct TerminalStyle {
    printable_string: String
}

impl Display for TerminalStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", esc_sq(self.printable_string.clone()))
    }
}

#[inline(always)]
fn concat_codes(code_string: &mut String, style: &String) {
    if !code_string.is_empty() {
        if !code_string.ends_with(';') {
            *code_string += ";";
        }
    }
    *code_string += style;
}

/// Builder for [`TerminalStyle`](struct@TerminalStyle).
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
        concat_codes(&mut self.style.printable_string, &color.fg_code());
        self
    }

    pub fn background(&mut self, color: Color) -> &mut Self {
        concat_codes(&mut self.style.printable_string, &color.bg_code());
        self
    }

    /// Add a style to text. Can be used multiple times.
    pub fn add_style(&mut self, style: Style) -> &mut Self {
        concat_codes(&mut self.style.printable_string, &style.code());
        self
    }

    /// Underline colors and style will only work on supported terminals.
    pub fn underline_style(&mut self, underline_style: UnderlineStyle) -> &mut Self {
        if is_underline_style_supported() {
            concat_codes(&mut self.style.printable_string, &underline_style.code());
        }
        self
    }

    /// Underline colors and style will only work on supported terminals.
    pub fn underline_color(&mut self, underline_color: Color) -> &mut Self {
        if is_underline_style_supported() {
            concat_codes(&mut self.style.printable_string, &underline_color.ul_code());
        }
        self
    }

    pub fn build(&mut self) -> TerminalStyle {
        self.style.clone()
    }
}

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
