use rusqlite::types::Value;

use tui::{
    layout::{Constraint, Layout, Direction, Rect},
    backend::Backend,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Table, Row, Cell, Borders, Block, Tabs, Clear},
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
    let dbs = qb.tabs
        .titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Spans::from(vec![
                Span::styled(first, Style::default().fg(Color::Yellow)),
                Span::styled(rest, Style::default()),
            ])
        }).collect();
    let ttabs = Tabs::new(dbs)
        .block(Block::default().borders(Borders::ALL))
        .select(qb.tabs.index)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        ;
    f.render_widget(ttabs, rect);
}

pub fn make_rows<B: Backend>(qb: &mut Qb, f: &mut Frame<B>, rect: Rect) {
    let headers_cells = qb.selected().scheme
        .iter()
        .skip(qb.selected().hstate)
        .map(|h| Cell::from(h.clone()).style(Style::default().add_modifier(Modifier::REVERSED)));
    let header_rows = Row::new(headers_cells)
        .height(1);
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let rows = qb.selected().items.iter().map(|item| {
        let height = item
            .iter()
            // .take(4)
            .map(|content| show(content).chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().skip(qb.selected().hstate).map(|c| Cell::from(show(c)));
        Row::new(cells).height(height as u16)
    });
    let mut cons = Vec::new();
    for _ in 0..qb.selected().hwidth {
        cons.push(Constraint::Percentage(20));
    }

    let t = Table::new(rows)
        // .block(Block::default())
        .header(header_rows)
        .highlight_style(selected_style)
        // We need to loop over rows etc depending on f.size and min(row.length, maxlength)
        .widths(&cons[..]);
    f.render_stateful_widget(t, rect, &mut qb.mutselected().state);
}

pub fn make_popup<B: Backend>(qb: &Qb, f: &mut Frame<B>) {
    match &qb.selected().zoom {
        Some(row) => {
            let block = Block::default().title("Zoom").borders(Borders::ALL);
            let area = centered_rect(100, 20, f.size());
            let text = row.iter().map(|c| { show(c)
            });
            f.render_widget(Clear, area); //this clears out the background
            f.render_widget(block, area);
        }
        None => {}
    }
}
