use rusqlite::{Connection, Result};
use std::env;
use rusqlite::types::Value;

use std::io;
use tui::{
    layout::{Constraint, Layout, Direction},
    backend::TermionBackend,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Table, TableState, Row, Cell, Borders, Block, Tabs},
    Terminal,
};
use termion::{raw::IntoRawMode, screen::AlternateScreen};

extern crate termion;
use termion::event::Key;
use termion::input::TermRead;

pub struct Qb<'a> {
    conn: &'a Connection,
    tabs: TabsState,
    dbs: Vec<Option<DbTable>>,
}

impl Qb <'_> {
    pub fn new(conn: &Connection) -> Qb {
        Qb {
            conn,
            tabs: TabsState::new(Vec::new()),
            dbs: Vec::new(),
        }
    }
    pub fn open(&mut self, db: String, index: usize) -> Result<()> {
            let ent = get_entries(&self.conn, db.clone());
            let (scheme, ents) = ent?;
            self.dbs[index] = Some(DbTable::new(db, scheme, ents));
        // }
        Ok(())
    }
    pub fn open_current(&mut self) -> Result<()> {
        self.open(self.tabs.get_tab(), self.tabs.index)
    }

    pub fn selected(&self) -> &DbTable {
        &self.dbs[self.tabs.index].as_ref().unwrap()
    }

    pub fn mutselected(&mut self) -> &mut DbTable {
        self.dbs[self.tabs.index].as_mut().unwrap()
    }

    fn get_names(&mut self) -> Result<()> {
        let mut stmt = self.conn.prepare("SELECT name FROM sqlite_master WHERE type ='table' AND name NOT LIKE 'sqlite_%';")?;
        let rows = stmt.query_map([], |row| row.get(0))?;

        let mut tbls = Vec::new();
        for tbl in rows {
            let t: String = tbl?;
            tbls.push(t.clone());
            // self.dbs.insert(t, None);
        }
        let size = tbls.len();
        self.tabs = TabsState::new(tbls);
        self.dbs = vec![None; size];
        Ok(())
    }   
}

fn get_entries(conn: &Connection, name: String) -> Result<(Vec<String>,Vec<Vec<Value>>)> {
    let sql = format!("SELECT * FROM {}", name);
    let mut stmt = conn.prepare(&sql)?;
    let value = stmt.column_names();
    let scheme: Vec<String> = value.iter().map(|s| s.to_string()).collect();
    let rows = stmt.query_map([], |row| {
        let mut cols = Vec::new();
        let ncols = row.column_count();
        for i in 0..ncols {
            cols.push(row.get(i)?)
        }
        Ok(cols)
    })?;

    let mut entries = Vec::new();
    for entry in rows {
        entries.push(entry?);
    }

    Ok((scheme, entries))
}

#[derive(Clone)]
pub struct DbTable {
    name: String,
    scheme: Vec<String>,
    state: TableState,
    items: Vec<Vec<Value>>,
    hstate: usize,
    hlen: usize,
    hwidth: usize,
}


impl DbTable {
    fn new(name: String, scheme: Vec<String>, db: Vec<Vec<Value>>) -> Self {
        let hlen = if db.len() < 1 {
            0
        } else {
            db[0].len()
        };
        DbTable {
            name,
            scheme,
            state: TableState::default(),
            items: db,
            hstate: 0,
            hlen,
            hwidth: 5,
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
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
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    // maybe these 2 should wrap
    pub fn hnext(&mut self) {
        if self.hlen > self.hwidth && self.hstate < self.hlen - self.hwidth {
            self.hstate += 1; 
        }
    }

    pub fn hprev(&mut self) {
        if self.hstate > 0 {
            self.hstate -= 1; 
        }
    }
}

pub struct TabsState {
    pub titles: Vec<String>,
    pub index: usize,
}

impl TabsState {
    pub fn new(titles: Vec<String>) -> TabsState {
        TabsState {
            titles, 
            index: 0,
        }
    }

    pub fn get_tab(&self) -> String {
        self.titles[self.index].clone()
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }
    pub fn prev(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }

    pub fn set(&mut self, i: usize) {
        self.index = i;
    }

    pub fn title_index(self, title: String) -> usize {
        let mut i = 0;
        for t in self.titles.iter() {
            if *t == title {
                break;
            }
            i += 1;
        }
        return i;
    }
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

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("syntax: qb file");
        return Ok(());
    }
    let path = &args[1];
    let conn = Connection::open(path)?;
    let mut qb = Qb::new(&conn);
    qb.get_names()?;
    qb.open_current()?;
 
    // Should seperate the draw functions
    let stdout = io::stdout().into_raw_mode().expect("Can't do raw mode");
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect("Backend failed");
    terminal.clear().expect("Clear error");

   
    'lp: loop {
        terminal.draw(|f| {
            let rect = Layout::default()
                .direction(Direction::Vertical)
                // maybe just a line?
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(f.size());
            let selected_style = Style::default().add_modifier(Modifier::REVERSED);
            let headers_cells = qb.selected().scheme
                .iter()
                .skip(qb.selected().hstate)
                .map(|h| Cell::from(h.clone()).style(Style::default().add_modifier(Modifier::REVERSED)));
            let header_rows = Row::new(headers_cells)
                .height(1);
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
            f.render_widget(ttabs, rect[0]);
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
           f.render_stateful_widget(t, rect[1], &mut qb.mutselected().state);
        }).expect("render error");
        let stdin = io::stdin();
        for c in stdin.keys() {
            let ch = c.unwrap();
            match ch {
                Key::Char('q') => {
                        break 'lp;
                }
                Key::Down | Key::Char('j') => {
                    qb.mutselected().next();
                    break;
                }
                Key::Up | Key::Char('k') => {
                    qb.mutselected().prev();
                    break;
                }
                Key::Left | Key::Char('h') => {
                    qb.mutselected().hprev();
                    break;
                }
                Key::Right | Key::Char('l') => {
                    qb.mutselected().hnext();
                    break;
                }
                Key::Char('n') => {
                    qb.tabs.next();
                    qb.open_current()?;
                    break;
                }
                Key::Char('p') => {
                    qb.tabs.prev();
                    qb.open_current()?;
                    break;
                }
                _ => {}
            }
        }
    }
    Ok(())
}
