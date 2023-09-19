// use tui_input::Input;

use std::fmt::Display;

use ratatui::{prelude::{Layout, Rect, Direction, Constraint, Backend}, Frame, widgets::{Paragraph, Block, Borders}};
use tui_input::InputRequest;

pub enum InputType {
    Exec,
    Query,
}

impl Display for InputType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputType::Exec => write!(f, "Exec"),
            InputType::Query => write!(f, "Query"),
        }
    }
}

pub struct Input {
    lines: u16,
    width_procent: u16,
    input: tui_input::Input,
    pub kind: InputType,
}

impl Input { 
    pub fn new(height: u16, width: u16, kind: InputType, value: String) -> Self {
        Input {
            lines:  height,
            width_procent: width,
            input: tui_input::Input::new(value),
            kind,
        }
    }
    pub fn set_input(&mut self, value: String) {
        self.input = tui_input::Input::new(value);
    }

    pub fn handle(&mut self, event: InputRequest) {
        self.input.handle(event);
    }

    pub fn render<B: Backend>(&self, f: &mut Frame<B>) { 
        let block = Block::default().title(self.kind.to_string()).borders(Borders::ALL);
        let area = centered_rect(self.width_procent, self.lines, f.size());

        let input = Paragraph::new(self.input.value())
            .block(block);
        f.render_widget(input, area);
    }
}

fn centered_rect(percent_x: u16, lines: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Length(lines),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
