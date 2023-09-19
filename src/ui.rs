use std::fmt::Display;

use rusqlite::types::Value;
use anyhow::Result;

use ratatui::{
    layout::{Constraint, Layout, Direction, Rect},
    backend::Backend,
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Borders, Block, Tabs},
    Frame
};
use crate::Qb;


fn show(v: &Value) -> String {
    match v {
        Value::Null => "Null".to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Real(f) => f.to_string(),
        Value::Text(t) => t.to_string(),
        Value::Blob(_b) => "Blob".to_string(),
    }
}

fn show_multiline(v: &Value, len: u16) -> String {
    match v {
        Value::Null => "Null".to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Real(f) => f.to_string(),
        Value::Text(text) => {
            let mut result = String::new();
            for (i, c) in text.chars().enumerate() {
                result.push(c);
                if (i + 1) % (len as usize) == 0 {
                    result.push('\n');
                }
            }
            result
        }
        Value::Blob(_b) => "Blob".to_string(),
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
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

pub fn make_tabs<B: Backend>(qb: &Qb, f: &mut Frame<B>, rect: Rect) {
    let dbs = qb.titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Line::from(vec![
            // Spans::from(vec![
                Span::styled(first, Style::default().fg(Color::Yellow)),
                Span::styled(rest, Style::default()),
            ])
        }).collect();
    let ttabs = Tabs::new(dbs)
        .block(Block::default().borders(Borders::ALL))
        .select(qb.index)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        ;
    f.render_widget(ttabs, rect);
}

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

// pub fn InputBox<B: Backend>(qb: &mut Qb, f: &mut Frame<B>, input: InputType) -> Result<()>{
//     let block = Block::default().title(input.show()).borders(Borders::ALL);
//     let area = centered_rect(100, 20, f.size());
//
//     Ok(())
// }

