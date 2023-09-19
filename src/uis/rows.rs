use ratatui::{widgets::{TableState, Cell, Row, Table}, prelude::{Backend, Constraint, Rect}, style::{Style, Modifier}, Frame};
use rusqlite::types::Value;

use super::zoom::Zoom;


#[derive(Clone)]
pub struct DbTable {
    pub scheme: Vec<String>,
    pub state: TableState,
    pub entries: Vec<Vec<Value>>,
    pub hstate: usize,
    pub hlen: usize,
    pub hwidth: usize,
    pub zoom: Zoom,
}

fn show(v: &Value) -> String {
    match v {
        Value::Null => "Null".to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Real(f) => f.to_string(),
        Value::Text(t) => t.to_string(),
        Value::Blob(_b) => "Blob".to_string(),
    }
}

impl DbTable {
    pub fn new(scheme: Vec<String>, entries: Vec<Vec<Value>>) -> Self {
        let hlen = if entries.is_empty() {
            0
        } else {
            entries[0].len()
        };
        let len = scheme.len();
        DbTable {
            scheme,
            state: TableState::default(),
            entries,
            hstate: 0,
            hlen,
            hwidth: usize::min(5, len),
            zoom: Zoom::new(100, 70, 5),
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.entries.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    pub fn prev(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    // lets wrap around
                    self.entries.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    pub fn set(&mut self, i: usize) {
        self.state.select(Some(i));
    }

    pub fn first(&mut self) {
        self.set(0);
    }

    pub fn grep(&mut self, ) {
    }
    
    pub fn last(&mut self) {
        self.set(self.entries.len() - 1);
    }

    // maybe these 2 should wrap
    pub fn hnext(&mut self) {
        self.hstate = usize::min(self.hstate + 1, self.hlen - self.hwidth)
    }

    pub fn hprev(&mut self) {
        if self.hstate > 0 {
            self.hstate -= 1; 
        }
    }
    
    pub fn render<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) {
        let headers_cells = self.scheme
            .iter()
            .skip(self.hstate)
            .map(|h| Cell::from(h.clone()).style(Style::default().add_modifier(Modifier::REVERSED)));
        let header_rows = Row::new(headers_cells)
            .height(1);
        let selected_style = Style::default().add_modifier(Modifier::REVERSED);
        let rows = self.entries.iter().map(|item| {
            let height = item
                .iter()
                // .take(4)
                .map(|content| show(content).chars().filter(|c| *c == '\n').count())
                .max()
                .unwrap_or(0)
                + 1;
            let cells = item.iter().skip(self.hstate).map(|c| Cell::from(show(c)));
            Row::new(cells).height(height as u16)
        });
        let mut cons = Vec::new();
        let width = 100/self.hwidth as u16;
        for _ in 0..self.hwidth {
            cons.push(Constraint::Percentage(width));
        }

        let t = Table::new(rows)
            // .block(Block::default())
            .header(header_rows)
            .highlight_style(selected_style)
            // We need to loop over rows etc depending on f.size and min(row.length, maxlength)
            .widths(&cons[..]);
        f.render_stateful_widget(t, rect, &mut self.state);
    }
}
