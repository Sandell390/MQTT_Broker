use std::fmt;

// ANSI escape sequences for text formatting
#[allow(dead_code)]
pub enum Style {
    Reset,
    Bold,
    Dim,
    Italic,
    Underline,
    Strikethrough,
    Blink,
    Reverse,
    Hidden,
}

// ANSI escape sequences for text colors
#[allow(dead_code)]
pub enum Color {
    Reset,
    Black,
    Red,
    Green,
    LimeGreen,
    Yellow,
    Blue,
    Magenta,
    Pink,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

// ANSI escape sequences for background colors
#[allow(dead_code)]
pub enum BackgroundColor {
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

// Enum variant for resetting text formatting attributes
#[allow(dead_code)]
pub enum Reset {
    TextStyle,
    TextColor,
    BackgroundColor,
    All,
    Default,
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Style::Reset => write!(f, "\x1b[22m"),
            Style::Bold => write!(f, "\x1b[1m"),
            Style::Dim => write!(f, "\x1b[2m"),
            Style::Italic => write!(f, "\x1b[3m"),
            Style::Underline => write!(f, "\x1b[4m"),
            Style::Strikethrough => write!(f, "\x1b[9m"),
            Style::Blink => write!(f, "\x1b[5m"),
            Style::Reverse => write!(f, "\x1b[7m"),
            Style::Hidden => write!(f, "\x1b[8m"),
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Color::Reset => write!(f, "\x1b[0m"),
            Color::Black => write!(f, "\x1b[30m"),
            Color::Red => write!(f, "\x1b[31m"),
            Color::Green => write!(f, "\x1b[32m"),
            Color::LimeGreen => write!(f, "\x1b[38;5;46m"),
            Color::Yellow => write!(f, "\x1b[33m"),
            Color::Blue => write!(f, "\x1b[34m"),
            Color::Magenta => write!(f, "\x1b[35m"),
            Color::Pink => write!(f, "\x1b[38;5;206m"),
            Color::Cyan => write!(f, "\x1b[36m"),
            Color::White => write!(f, "\x1b[37m"),
            Color::BrightBlack => write!(f, "\x1b[90m"),
            Color::BrightRed => write!(f, "\x1b[91m"),
            Color::BrightGreen => write!(f, "\x1b[92m"),
            Color::BrightYellow => write!(f, "\x1b[93m"),
            Color::BrightBlue => write!(f, "\x1b[94m"),
            Color::BrightMagenta => write!(f, "\x1b[95m"),
            Color::BrightCyan => write!(f, "\x1b[96m"),
            Color::BrightWhite => write!(f, "\x1b[97m"),
        }
    }
}

impl fmt::Display for BackgroundColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            BackgroundColor::Reset => write!(f, "\x1b[0m"),
            BackgroundColor::Black => write!(f, "\x1b[40m"),
            BackgroundColor::Red => write!(f, "\x1b[41m"),
            BackgroundColor::Green => write!(f, "\x1b[42m"),
            BackgroundColor::Yellow => write!(f, "\x1b[43m"),
            BackgroundColor::Blue => write!(f, "\x1b[44m"),
            BackgroundColor::Magenta => write!(f, "\x1b[45m"),
            BackgroundColor::Cyan => write!(f, "\x1b[46m"),
            BackgroundColor::White => write!(f, "\x1b[47m"),
            BackgroundColor::BrightBlack => write!(f, "\x1b[100m"),
            BackgroundColor::BrightRed => write!(f, "\x1b[101m"),
            BackgroundColor::BrightGreen => write!(f, "\x1b[102m"),
            BackgroundColor::BrightYellow => write!(f, "\x1b[103m"),
            BackgroundColor::BrightBlue => write!(f, "\x1b[104m"),
            BackgroundColor::BrightMagenta => write!(f, "\x1b[105m"),
            BackgroundColor::BrightCyan => write!(f, "\x1b[106m"),
            BackgroundColor::BrightWhite => write!(f, "\x1b[107m"),
        }
    }
}

impl fmt::Display for Reset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Reset::TextStyle => write!(f, "\x1b[0m"),
            Reset::TextColor => write!(f, "\x1b[39m"),
            Reset::BackgroundColor => write!(f, "\x1b[49m"),
            Reset::All => write!(f, "\x1b[0m\x1b[39m\x1b[49m"),
            Reset::Default => write!(f, "\x1b[0m\x1b[39m\x1b[49m\x1b[92m"),
        }
    }
}
