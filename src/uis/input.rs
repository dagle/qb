// use tui_input::Input;

use std::{fmt::Display, convert::TryFrom};

use ratatui::{prelude::{Rect, Backend}, Frame, widgets::Paragraph};
use tui_input::InputRequest;

use thiserror::Error;

// TDOO: This won't be a problem after refactor
#[derive(Error, Debug)]
pub enum ConvertError {
    #[error("Convertion error: report upstream")]
    ConvertError,

    #[error("Not a valid command: {0}")]
    InputTypeError(String),
}


pub enum InputType {
    Exec,
    Query,
}

impl Display for InputType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputType::Exec => write!(f, "exec"),
            InputType::Query => write!(f, "query"),
        }
    }
}

impl TryFrom<&str> for InputType {
    type Error = ConvertError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "exec" => Ok(InputType::Exec),
            "query" => Ok(InputType::Query),
            _ => Err(ConvertError::InputTypeError(value.to_owned())),
        }
    }
}

pub struct Input {
    pub input: tui_input::Input,
    pub kind: InputType,
}

impl Input { 
    pub fn new(kind: InputType, value: String) -> Self {
        Input {
            input: tui_input::Input::new(value),
            kind,
        }
    }

    pub fn handle(&mut self, event: InputRequest) {
        self.input.handle(event);
    }

    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) { 
        let input = Paragraph::new(self.input.value());
        f.render_widget(input, area);
        let x = area.x + self.input.visual_cursor() as u16;
        let y = area.y;
        f.set_cursor(x, y+1);
    }
}
