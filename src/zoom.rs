use ratatui::{prelude::{Backend, Direction, Constraint, Layout, Rect}, Frame, widgets::{Block, Borders, Row, Cell, Table}, style::{Style, Modifier}};
use rusqlite::types::Value;

use super::rows::DbTable;

#[derive(Clone)]
pub struct Zoom {
    height_procent: u16,
    width_procent: u16,
    zoom_width: usize,
    hstate: usize,
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

// TODO: move this to generic thing
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

impl Zoom {
    pub fn new(width: u16, height: u16, zoom_width: usize) -> Self {
        Zoom {
            height_procent: height,
            width_procent: width,
            zoom_width,
            hstate: 0,
        }
    }

    pub fn zoom_out(&mut self, max: usize) {
        self.zoom_width = usize::min(max, self.zoom_width + 1);
    }

    pub fn zoom_in(&mut self) {
        self.zoom_width = usize::max(1, self.zoom_width - 1);
    }

    pub fn next(&mut self, max: usize) {
        self.hstate = usize::min(self.hstate + 1, max - self.zoom_width); 
    }

    pub fn prev(&mut self) {
        if self.hstate > 0 {
            self.hstate -= 1;
        }
    }
    pub fn render<B: Backend>(&self, selected: &DbTable, f: &mut Frame<B>) { 
        if let Some(row) = selected.state.selected() {
            let block = Block::default().title("Zoom").borders(Borders::ALL);
            let area = centered_rect(self.width_procent, self.height_procent, f.size());

            let headers_cells = selected.scheme
                .iter()
                .skip(self.hstate)
                .map(|h| Cell::from(h.clone()).style(Style::default().add_modifier(Modifier::REVERSED)));
            let header_rows = Row::new(headers_cells)
                .height(1);
            let selected_style = Style::default().add_modifier(Modifier::REVERSED);
            let row = &selected.entries[row];
            let celllen = area.width / (self.zoom_width as u16);
            let values = vec![Row::new(row.iter().skip(self.hstate).map(|item| {
                Cell::from(show_multiline(item, celllen)) })).height(area.height-3)];
            let mut cons = Vec::new();
            let width = 100/self.zoom_width as u16;
            for _ in 0..self.zoom_width {
                cons.push(Constraint::Percentage(width));
            }

            let t = Table::new(values)
                .block(block)
                .header(header_rows)
                .highlight_style(selected_style)
                .widths(&cons[..]);
            f.render_widget(t, area);
        }
    }
}
