//! Enums for ANSI terminal cursor manipulation that all implement `Display`.

use std::fmt::Display;

const ESC: &str = "\u{001b}[";

trait HasCode {
    fn code(&self) -> String;
}

impl Display for dyn HasCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", ESC, self.code())
    }
}

/// Move the cursor.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Cursor {
    /// Save cursor position (SCO).
    Save,
    /// Restore cursor position (SCO).
    Restore,
    /// Position 0, 0.
    Reset,
    /// One line up.
    Up,
    /// One line down.
    Down,
    Right(u32),
    Left(u32),
    /// Column N.
    Position(u32),
}

impl HasCode for Cursor {
    fn code(&self) -> String {
        match self {
            Cursor::Save            => "s".to_string(),
            Cursor::Restore         => "u".to_string(),
            Cursor::Reset           => "H".to_string(),
            Cursor::Up              => "1A".to_string(),
            Cursor::Down            => "1B".to_string(),
            Cursor::Right(value)    => format!("{}C", value),
            Cursor::Left(value)     => format!("{}D", value),
            Cursor::Position(value) => format!("{}G", value),
        }
    }
}

impl Display for Cursor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", ESC, self.code())
    }
}

/// Jump to the beginning of lines.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Line {
    /// First character of the current line.
    Home,
    /// Beginning of the next line.
    Next,
    /// Beginning of the previous line.
    Previous,
    Up(u32),
    Down(u32),
}

impl HasCode for Line {
    fn code(&self) -> String {
        match self {
            Line::Home        => "0G".to_string(),
            Line::Next        => "1E".to_string(),
            Line::Previous    => "1F".to_string(),
            Line::Up(value)   => format!("{}E", value),
            Line::Down(value) => format!("{}F", value),
        }
    }
}

impl Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", ESC, self.code())
    }
}

/// Screen erasing.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Erase {
    Screen,
    /// Everything after.
    AllAfter,
    /// Everything before.
    AllBefore,
    Line,
    /// Everything after in a line.
    LineAfter,
    /// Everything before in a line.
    LineBefore,
}

impl HasCode for Erase {
    fn code(&self) -> String {
        match self {
            Erase::Screen     => "2J".to_string(),
            Erase::AllAfter   => "0J".to_string(),
            Erase::AllBefore  => "1J".to_string(),
            Erase::Line       => "2K".to_string(),
            Erase::LineAfter  => "0K".to_string(),
            Erase::LineBefore => "1K".to_string(),
        }
    }
}

impl Display for Erase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", ESC, self.code())
    }
}
