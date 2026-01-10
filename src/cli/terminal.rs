use std::fmt;
use std::io::{self, Write};
use crate::cli::layout::DisplayLayout;

#[derive(Debug, PartialEq)]
pub enum AnsiSequence {
    SaveCursor,      // \x1b[s
    RestoreCursor,   // \x1b[u
    HideCursor,      // \x1b[?25l
    ShowCursor,      // \x1b[?25h
    MoveUp(u16),     // \x1b[nA
    ClearLine,       // \x1b[2K
}

impl fmt::Display for AnsiSequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // TODO: 正しい実装にする
            AnsiSequence::SaveCursor => write!(f, ""),
            AnsiSequence::RestoreCursor => write!(f, ""),
            AnsiSequence::HideCursor => write!(f, ""),
            AnsiSequence::ShowCursor => write!(f, ""),
            AnsiSequence::MoveUp(_) => write!(f, ""),
            AnsiSequence::ClearLine => write!(f, ""),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_sequence_display() {
        assert_eq!(AnsiSequence::SaveCursor.to_string(), "\x1b[s");
        assert_eq!(AnsiSequence::RestoreCursor.to_string(), "\x1b[u");
        assert_eq!(AnsiSequence::HideCursor.to_string(), "\x1b[?25l");
        assert_eq!(AnsiSequence::ShowCursor.to_string(), "\x1b[?25h");
        assert_eq!(AnsiSequence::MoveUp(3).to_string(), "\x1b[3A");
        assert_eq!(AnsiSequence::ClearLine.to_string(), "\x1b[2K");
    }
}
