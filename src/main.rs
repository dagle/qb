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
use std::collections::HashMap;

// use crate::util::event::{Event, Events};
// use std::sync::mpsc;

extern crate termion;
use termion::event::Key;
use termion::input::TermRead;

// use rusqlite::NO_PARAMS;

pub struct Qb {
    con: Connection, 
    tabs: TabsState,
    dbs: HashMap<String, DbTable>,
}

impl Qb {
    pub fn new(con: Connection, tabs: TabsState) -> Qb {
        Qb {
            con,
            tabs,
            dbs: HashMap::new(),
        }
    }
    pub fn open(&mut self, db: String) {
        if (!self.dbs.contains_key(db)) {
            // open and read the db if it's not in memory
        }
        self.tabs.
    }
}

pub struct DbTable {
    state: TableState,
    items: Vec<Vec<Value>>,
    hstate: usize,
    hlen: usize,
    hwidth: usize,
}


impl DbTable {
    fn new(db: Vec<Vec<Value>>) -> Self {
        let hlen = if db.len() < 1 {
            0
        } else {
            db[0].len()
        };
        DbTable {
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
        if self.hstate < self.hlen - self.hwidth {
            self.hstate += 1; 
        }
    }

    pub fn hprev(&mut self) {
        if self.hstate > 0 {
            self.hstate -= 1; 
        }
    }

    // pub fn scroll(&mut self) -> usize {
    //     let i = self.state.selected().unwrap_or_else(||0);
    //         if i < 83 {
    //             0
    //         } else {
    //             i - 83
    //         }
    // }
            // let scrolls = table.state.selected().unwrap_or_else(||0);
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

    pub fn get_tab(self) -> String {
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

    pub fn set_title(&mut self, title: String) {
        let i = self.title_index(title);
        self.index = i;
    }
    // maybe use a easier datatype to solve this
    // Should return an option
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

fn get_names(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type ='table' AND name NOT LIKE 'sqlite_%';")?;
    let rows = stmt.query_map([], |row| row.get(0))?;

    let mut tbls = Vec::new();
    for tbl in rows {
        tbls.push(tbl?);
    }

    Ok(tbls)
}

// fn get__scheme(conn: &Connection, name: String) -> Result<Vec<String>> {
//     let sql = format!("SELECT sql FROM sqlite_master WHERE name = '{}';", name);
//     let mut stmt = conn.prepare(&sql)?;
//     let rows = stmt.query_map([], |row| row.get(0))?;

//     let mut tbls = Vec::new();
//     for tbl in rows {
//         tbls.push(tbl?);
//     }

//     Ok(tbls)
// }

fn get_scheme(conn: &Connection, name: String) -> Result<Vec<String>> {
    let sql = format!("SELECT * FROM {}", name);
    let stmt = conn.prepare(&sql)?;
    let value = stmt.column_names();
    Ok(value.iter().map(|s| s.to_string()).collect())
}

// maybe it should do some lazy loading but I think limit shouldn't be visable
fn get_entries(conn: &Connection, name: String) -> Result<Vec<Vec<Value>>> {
    let sql = format!("SELECT * FROM {}", name);
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
        let mut cols = Vec::new();
        let ncols = row.column_count();
        for i in 0..ncols {
            cols.push(row.get(i)?)
        }
        Ok(cols)
    })?;
    // stmt.column_count

    let mut entries = Vec::new();
    for entry in rows {
        entries.push(entry?);
    }

    Ok(entries)
}

// fn parse_sql(sql: String) { // -> Result<Vec<Statement>, ParserError> {
//     let dialect = GenericDialect {};
//     let asts = Parser::parse_sql(&dialect, &sql).unwrap();
//     // println!("{:?}", asts);
//     for ast in asts {
//         match ast {
//             Statement::CreateTable {name, columns, ..} => { 
//                 println!("table: {}", name);
//                 for column in columns {
//                     println!("column: {} {}", column.name, column.data_type);
//                 }
//             },
//             _ => println!("no match"),
//         }
//     }
// }

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
    // real arg parsing in the future
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("syntax: qb file");
        return Ok(());
    }

    let stdout = io::stdout().into_raw_mode().expect("Can't do raw mode");
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect("Backend failed");
    terminal.clear().expect("Clear error");

    let path = &args[1];
    let conn = Connection::open(path)?;
    let tbls = get_names(&conn)?;

    let mut tabs = TabsState::new(tbls.clone());
    let entries = get_entries(&conn, tbls[0].clone())?;
    let headers = get_scheme(&conn, tbls[0].clone())?;
    // let entries = get_entries(&conn, "moz_cookies".to_string())?;
    // let headers = get_scheme(&conn, "moz_cookies".to_string())?;

    let mut table = DbTable::new(entries.clone());

    'lp: loop {
        terminal.draw(|f| {
            let rect = Layout::default()
                .direction(Direction::Vertical)
                // maybe just a line?
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(f.size());
            let selected_style = Style::default().add_modifier(Modifier::REVERSED);
            let headers_cells = headers
                .iter()
                .skip(table.hstate)
                .map(|h| Cell::from(h.clone()).style(Style::default().add_modifier(Modifier::REVERSED)));
            let header_rows = Row::new(headers_cells)
                .height(1);
            let dbs = tabs
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
                .select(tabs.index)
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                ;
            f.render_widget(ttabs, rect[0]);
            let rows = table.items.iter().map(|item| {
                let height = item
                    .iter()
                    // .take(4)
                    .map(|content| show(content).chars().filter(|c| *c == '\n').count())
                    .max()
                    .unwrap_or(0)
                    + 1;
                let cells = item.iter().skip(table.hstate).map(|c| Cell::from(show(c)));
                Row::new(cells).height(height as u16)
            });
            let mut cons = Vec::new();
            // for _ in rows.clone() {
            for _ in 0..table.hwidth {
                cons.push(Constraint::Percentage(20));
            }

            let t = Table::new(rows)
                // .block(Block::default())
                .header(header_rows)
                .highlight_style(selected_style)
                // .highlight_symbol(">>")
                // We need to loop over rows etc depending on f.size and min(row.length, maxlength)
                .widths(&cons[..]);
           f.render_stateful_widget(t, rect[1], &mut table.state);
        }).expect("render error");
        let stdin = io::stdin();
        for c in stdin.keys() {
            let ch = c.unwrap();
            match ch {
                Key::Char('q') => {
                        break 'lp;
                }
                Key::Down | Key::Char('j') => {
                    table.next();
                    break;
                    // println!("{:?}", table.state.selected());
                }
                Key::Up | Key::Char('k') => {
                    table.prev();
                    break;
                }
                Key::Left | Key::Char('h') => {
                    table.hprev();
                    break;
                }
                Key::Right | Key::Char('l') => {
                    table.hnext();
                    break;
                }
                _ => {}
            }
        }
    }
    Ok(())
}
