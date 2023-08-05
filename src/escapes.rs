//! Most common escapes to manipulate terminals.
//! All enums implement [`Display`] trait.

use std::fmt::Display;

const ESC: &str = "\u{001b}";

trait HasCode {
    fn code(&self) -> String;
}

/// System escapes.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum System<'a> {
    ResetState,
    SaveState,
    RestoreState,
    SetTitle(&'a str),
}

impl<'a> HasCode for System<'a> {
    fn code(&self) -> String {
        match self {
            System::ResetState      => "c".to_string(),
            System::SaveState       => "7".to_string(),
            System::RestoreState    => "8".to_string(),
            System::SetTitle(value) => format!("]0;{}\u{0007}", value),
        }
    }
}

const CSI: &str = "\u{001b}[";

impl<'a> Display for System<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if cfg!(feature = "mock_codes") {
            write!(f, "{{code {}}}", self.code())
        } else {
            write!(f, "{}{}", ESC, self.code())
        }
    }
}

/// Cursor movement.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Cursor {
    /// Save cursor position (SCO).
    Save,
    /// Restore cursor position (SCO).
    Restore,
    /// Position 0, 0.
    Reset,
    Up(u32),
    Down(u32),
    Right(u32),
    Left(u32),
    /// Move to column N.
    Column(u8),
    /// Set absolute position.
    Goto(u32, u32),
}

impl HasCode for Cursor {
    fn code(&self) -> String {
        match self {
            Cursor::Save            => "s".to_string(),
            Cursor::Restore         => "u".to_string(),
            Cursor::Reset           => "H".to_string(),
            Cursor::Up(value)       => format!("{}A", value),
            Cursor::Down(value)     => format!("{}B", value),
            Cursor::Right(value)    => format!("{}C", value),
            Cursor::Left(value)     => format!("{}D", value),
            Cursor::Column(value)   => format!("{}G", value),
            Cursor::Goto(x, y)      => format!("{};{}H", x, y),
        }
    }
}

impl Display for Cursor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if cfg!(feature = "mock_codes") {
            write!(f, "{{code {}}}", self.code())
        } else {
            write!(f, "{}{}", CSI, self.code())
        }
    }
}

/// Jumps to beginning of lines.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Line {
    /// First character of the current line.
    Home,
    Up(u32),
    Down(u32),
}

impl HasCode for Line {
    fn code(&self) -> String {
        match self {
            Line::Home        => "0G".to_string(),
            Line::Up(value)   => format!("{}E", value),
            Line::Down(value) => format!("{}F", value),
        }
    }
}

impl Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if cfg!(feature = "mock_codes") {
            write!(f, "{{code {}}}", self.code())
        } else {
            write!(f, "{}{}", CSI, self.code())
        }
    }
}

/// Screen erasing.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Erase {
    /// Insert N blank characters.
    After(u32),
    Screen,
    /// Whole screen, including scroll buffer.
    WholeScreen,
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
            Erase::After(value) => format!("{}@", value),
            Erase::Screen       => "2J".to_string(),
            Erase::WholeScreen  => "3J".to_string(),
            Erase::AllAfter     => "0J".to_string(),
            Erase::AllBefore    => "1J".to_string(),
            Erase::Line         => "2K".to_string(),
            Erase::LineAfter    => "0K".to_string(),
            Erase::LineBefore   => "1K".to_string(),
        }
    }
}

impl Display for Erase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if cfg!(feature = "mock_codes") {
            write!(f, "{{code {}}}", self.code())
        } else {
            write!(f, "{}{}", CSI, self.code())
        }
    }
}
