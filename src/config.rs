use std::{collections::HashMap, fmt::Display, convert::TryInto};

use qb::error::ConvertError;
use ratatui::style::Color;
use serde::{Deserialize, Serialize, Serializer, ser::SerializeTuple};
use crossterm::event::{KeyCode, ModifierKeyCode, KeyEvent, KeyEventState, KeyEventKind, KeyModifiers};
use tui_input::InputRequest;

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
    #[serde(serialize_with = "ColorDef::ser_rgb")]
    Rgb(u8, u8, u8),
    Indexed(u8),
}

impl ColorDef {
    fn ser_rgb<S>(r: &u8, g: &u8, b: &u8, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer {
            let mut tup = s.serialize_tuple(3)?;
            tup.serialize_element(r)?;
            tup.serialize_element(g)?;
            tup.serialize_element(b)?;
            tup.end()
    }
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
pub enum MainAction {
    Next,
    Prev,
    Hnext,
    Hprev,
    Tnext,
    Tprev,
    First,
    Last,
    Zoom,
    InputCurrent(String),
    Input(String),
    Quit,
    Search,
    Reload,
    // ClearError,
}

#[derive(Serialize, Deserialize)]
pub enum ZoomAction {
    Back,
    ZoomIn,
    ZoomOut,
    Next,
    Prev,
}

#[derive(Serialize, Deserialize)]
pub enum InputAction {
    GoToPrevChar,
    GoToNextChar,
    GoToPrevWord,
    GoToNextWord,
    GoToStart,
    GoToEnd,
    DeletePrevChar,
    DeleteNextChar,
    DeletePrevWord,
    DeleteNextWord,
    DeleteLine,
    DeleteTillEnd,
    Enter,
    Leave,
}

impl TryInto<InputRequest> for &InputAction {
    type Error = ConvertError;

    fn try_into(self) -> Result<InputRequest, Self::Error> {
        match self {
            InputAction::GoToPrevChar => Ok(InputRequest::GoToPrevChar),
            InputAction::GoToNextChar => Ok(InputRequest::GoToNextChar),
            InputAction::GoToPrevWord => Ok(InputRequest::GoToPrevWord),
            InputAction::GoToNextWord => Ok(InputRequest::GoToNextWord),
            InputAction::GoToStart => Ok(InputRequest::GoToStart),
            InputAction::GoToEnd => Ok(InputRequest::GoToEnd),
            InputAction::DeletePrevChar => Ok(InputRequest::DeletePrevChar),
            InputAction::DeleteNextChar => Ok(InputRequest::DeleteNextChar),
            InputAction::DeletePrevWord => Ok(InputRequest::DeletePrevWord),
            InputAction::DeleteNextWord => Ok(InputRequest::DeleteNextWord),
            InputAction::DeleteLine => Ok(InputRequest::DeleteLine),
            InputAction::DeleteTillEnd => Ok(InputRequest::DeleteTillEnd),
            InputAction::Enter => Err(ConvertError::ConvertError),
            InputAction::Leave => Err(ConvertError::ConvertError),
        }
    }
}

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq, Clone, Copy)]
pub enum Mode {
    Main,
    Zoom,
    Input,
    Visual,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Main => write!(f, "Main"),
            Mode::Zoom => write!(f, "Zoom"),
            Mode::Input => write!(f, "Input"),
            Mode::Visual => write!(f, "Visual"),
        }
    }
}

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct KeyBind {
    pub key: KeyCode,
    pub modifier: Option<ModifierKeyCode>,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub colors: Colors,
    // pub keybinds: HashMap<KeyEvent, Action>,
    // pub keymodes: HashMap<Mode, Keymode>,
    pub main: HashMap<KeyEvent, MainAction>,
    pub zoom: HashMap<KeyEvent, ZoomAction>,
    pub input: HashMap<KeyEvent, InputAction>
}

macro_rules! keypress {
    ($keycode:expr,$modifier:expr) => {
        KeyEvent { code: $keycode, modifiers: $modifier, kind: KeyEventKind::Press,  state: KeyEventState::NONE }
    };
}

impl Default for Config {
    fn default() -> Self { 
        let mut main = HashMap::new();
        main.insert(keypress!(KeyCode::Up, KeyModifiers::NONE), MainAction::Prev);
        main.insert(keypress!(KeyCode::Char('k'), KeyModifiers::NONE), MainAction::Prev);
        main.insert(keypress!(KeyCode::Down, KeyModifiers::NONE), MainAction::Next);
        main.insert(keypress!(KeyCode::Char('j'), KeyModifiers::NONE), MainAction::Next);
        main.insert(keypress!(KeyCode::Left, KeyModifiers::NONE), MainAction::Hprev);
        main.insert(keypress!(KeyCode::Char('h'), KeyModifiers::NONE), MainAction::Hprev);
        main.insert(keypress!(KeyCode::Right, KeyModifiers::NONE), MainAction::Hnext);
        main.insert(keypress!(KeyCode::Char('l'), KeyModifiers::NONE), MainAction::Hnext);
        main.insert(keypress!(KeyCode::Home, KeyModifiers::NONE), MainAction::First);
        main.insert(keypress!(KeyCode::Char('g'), KeyModifiers::NONE), MainAction::First);
        main.insert(keypress!(KeyCode::End, KeyModifiers::NONE), MainAction::Last);
        main.insert(keypress!(KeyCode::Char('G'), KeyModifiers::NONE), MainAction::Last);
        main.insert(keypress!(KeyCode::Char('n'), KeyModifiers::CONTROL), MainAction::Tnext);
        main.insert(keypress!(KeyCode::Char('p'), KeyModifiers::CONTROL), MainAction::Tprev);
        main.insert(keypress!(KeyCode::Char('q'), KeyModifiers::NONE), MainAction::Quit);
        main.insert(keypress!(KeyCode::Char('r'), KeyModifiers::NONE), MainAction::Reload);
        main.insert(keypress!(KeyCode::Char('z'), KeyModifiers::NONE), MainAction::Zoom);
        main.insert(keypress!(KeyCode::Char('i'), KeyModifiers::NONE), MainAction::InputCurrent("query".to_owned()));
        main.insert(keypress!(KeyCode::Char('e'), KeyModifiers::NONE), MainAction::Input("exec ".to_owned()));

        let mut zoom = HashMap::new();
        zoom.insert(keypress!(KeyCode::Esc, KeyModifiers::NONE), ZoomAction::Back);
        zoom.insert(keypress!(KeyCode::Char('q'), KeyModifiers::NONE), ZoomAction::Back);
        zoom.insert(keypress!(KeyCode::Char('+'), KeyModifiers::NONE), ZoomAction::ZoomIn);
        zoom.insert(keypress!(KeyCode::Char('-'), KeyModifiers::NONE), ZoomAction::ZoomOut);
        zoom.insert(keypress!(KeyCode::Left, KeyModifiers::NONE), ZoomAction::Prev);
        zoom.insert(keypress!(KeyCode::Char('h'), KeyModifiers::NONE), ZoomAction::Prev);
        zoom.insert(keypress!(KeyCode::Right, KeyModifiers::NONE), ZoomAction::Next);
        zoom.insert(keypress!(KeyCode::Char('l'), KeyModifiers::NONE), ZoomAction::Prev);

        let mut input = HashMap::new();
        input.insert(keypress!(KeyCode::Esc, KeyModifiers::NONE), InputAction::Leave);
        input.insert(keypress!(KeyCode::Enter, KeyModifiers::NONE), InputAction::Enter);
        input.insert(keypress!(KeyCode::Backspace, KeyModifiers::NONE), InputAction::DeletePrevChar);
        input.insert(keypress!(KeyCode::Delete, KeyModifiers::NONE), InputAction::DeleteNextChar);
        input.insert(keypress!(KeyCode::Left, KeyModifiers::NONE), InputAction::GoToPrevChar);
        input.insert(keypress!(KeyCode::Right, KeyModifiers::NONE), InputAction::GoToNextChar);
        input.insert(keypress!(KeyCode::Char('a'), KeyModifiers::CONTROL), InputAction::GoToStart);
        input.insert(keypress!(KeyCode::Char('e'), KeyModifiers::CONTROL), InputAction::GoToEnd);
        input.insert(keypress!(KeyCode::Char('w'), KeyModifiers::CONTROL), InputAction::DeletePrevWord);
        
        Self { 
            colors: Colors::default(),
            main,
            zoom,
            input
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn test_serialize() {
        let _ = Config::default();
    }
}
