use crate::Qb;
use tui::style::{Color};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor};
use termion::event::Key;

#[derive(Serialize, Deserialize)]
#[serde(remote = "Color")]
enum ColorDef{
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
    Rgb(u8, u8, u8),
    Indexed(u8),
}

#[derive(Serialize, Deserialize)]
pub struct Colors {
    // maybe change to a style
    #[serde(with = "ColorDef")]
    tabcolor: Color,
    #[serde(with = "ColorDef")]
    hltab: Color,
    #[serde(with = "ColorDef")]
    headers: Color,
    #[serde(with = "ColorDef")]
    hlrow: Color,
}

impl Default for Colors {
    fn default() -> Self { 
        Self { 
            tabcolor: Color::Yellow,
            hltab: Color::Yellow,
            headers: Color::Yellow,
            hlrow: Color::Yellow,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum Action {
    Next,
    Prev,
    Hnext,
    Hprev,
    First,
    Last,
    Zoom,
    Quit,
    Search,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "Key")]
enum KeyDef {
    Backspace,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    BackTab,
    Delete,
    Insert,
    F(u8),
    Char(char),
    Alt(char),
    Ctrl(char),
    Null,
    Esc,

    #[doc(hidden)]
    __IsNotComplete,
}


#[derive(Serialize, Deserialize)]
pub struct KeyBind {
    #[serde(with = "KeyDef")]
    pub key: Key,
    pub action: Action
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub colors: Colors,
    pub keybinds: Vec<KeyBind>
}


impl Default for Config {
    fn default() -> Self { 
        Self { 
            keybinds: vec![
                KeyBind {key:Key::Up, action: Action::Prev},
                KeyBind {key:Key::Char('k'), action: Action::Prev},
                KeyBind {key:Key::Down, action: Action::Next},
                KeyBind {key:Key::Char('j'), action: Action::Next},
                KeyBind {key:Key::Left, action: Action::Hprev},
                KeyBind {key:Key::Char('h'), action: Action::Hprev},
                KeyBind {key:Key::Right, action: Action::Hnext},
                KeyBind {key:Key::Char('l'), action: Action::Hnext},
                KeyBind {key:Key::Home, action: Action::First},
                KeyBind {key:Key::Char('g'), action: Action::First},
                KeyBind {key:Key::End, action: Action::Last},
                KeyBind {key:Key::Char('G'), action: Action::Last},
                KeyBind {key:Key::Char('q'), action: Action::Quit},
            ],
            colors: Colors::default(),
        }
    }
}