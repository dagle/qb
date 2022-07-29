use rusqlite::{Connection, Result};
use rusqlite::types::Value;
use clap::Parser;
use std::path::PathBuf;
use tui::style::{Color, Modifier, Style};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor};

use std::io;

mod ui;
mod config;

use config::*;

use tui::{
    layout::{Constraint, Layout, Direction},
    backend::TermionBackend,
    widgets::TableState, 
    Terminal
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
            let ent = get_entries(self.conn, db.clone());
            let (scheme, ents) = ent?;
            self.dbs[index] = Some(DbTable::new(db, scheme, ents));
        // }
        Ok(())
    }
    pub fn open_current(&mut self) -> Result<()> {
        self.open(self.tabs.get_tab(), self.tabs.index)
    }

    pub fn selected(&self) -> &DbTable {
        self.dbs[self.tabs.index].as_ref().unwrap()
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
    zoom: Option<Vec<Value>>, // should be a ref
}


impl DbTable {
    fn new(name: String, scheme: Vec<String>, db: Vec<Vec<Value>>) -> Self {
        let hlen = if db.is_empty() {
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
            zoom: None,
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
    pub fn set(&mut self, i: usize) {
        self.state.select(Some(i));
    }

    pub fn first(&mut self) {
        self.set(0);
    }

    pub fn grep(&mut self, ) {
    }
    
    pub fn last(&mut self) {
        self.set(self.items.len() - 1);
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
        i
    }
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    db_path: PathBuf,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let cfg: Config = confy::load("qb", None)
        .expect("Couldn't load config file, remove it to get a new one");
    confy::store("qb", None, &cfg).expect("Couldn't update config");

    let path = &args.db_path;
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
            ui::make_tabs(&qb, f, rect[0]);
            ui::make_rows(&mut qb, f, rect[1]);
            ui::make_popup(&qb, f)
        }).expect("render error");
        let stdin = io::stdin();
        if let Some(c) = stdin.keys().next() {
            let ch = c.unwrap();
            if ch == Key::Char('q') {
                break 'lp;
            }
            for bind in &cfg.keybinds {
                if bind.key == ch {
                    match bind.action {
                        Action::Next => {
                            qb.mutselected().next();
                        }
                        Action::Prev => {
                            qb.mutselected().prev();
                        }
                        Action::Hnext => {
                            qb.mutselected().hnext();
                        }
                        Action::Hprev => {
                            qb.mutselected().hprev();
                        }
                        Action::First => {
                            qb.mutselected().first();
                        }
                        Action::Last => {
                            qb.mutselected().last();
                        }
                        Action::Zoom => {
                        }
                        Action::Quit => {
                            break 'lp;
                        }
                        Action::Search => {
                        }
                    }
                }
            }
            // for bind in keys.iter() {
            //     if ch == bind.key {
            //         (bind.fun)(&mut qb)
            //     }
            // }
            // match ch {
            //     Key::Char('q') => {
            //         break 'lp;
            //     }
            //     Key::Down | Key::Char('j') => {
            //         qb.mutselected().next();
            //     }
            //     Key::Up | Key::Char('k') => {
            //         qb.mutselected().prev();
            //     }
            //     Key::Left | Key::Char('h') => {
            //         qb.mutselected().hprev();
            //     }
            //     Key::Right | Key::Char('l') => {
            //         qb.mutselected().hnext();
            //     }
            //     Key::Home | Key::Char('g') => {
            //         qb.mutselected().first();
            //     }
            //     Key::End | Key::Char('G') => {
            //         qb.mutselected().last();
            //     }
            //     Key::Char('s') => {
            //         // get string
            //         // qb.mutselected().search(string)
            //     }
            //     Key::Char('\n') => {
            //         let selected = qb.mutselected();
            //         let zoom = &selected.zoom;
            //         match zoom {
            //             None => {
            //                 if let Some(i) = selected.state.selected() {
            //                     selected.zoom = Some(selected.items[i].clone());
            //                 }
            //             }
            //             _ => {
            //                 selected.zoom = None;
            //             }
            //         }
            //     }
            //     Key::Char('/') => {
            //         // get string
            //         // qb.mutselected().grep(string)
            //     }
            //     Key::Char('n') => {
            //         qb.tabs.next();
            //         qb.open_current()?;
            //     }
            //     Key::Char('p') => {
            //         qb.tabs.prev();
            //         qb.open_current()?;
            //     }
            //     _ => {}
            // }
        }
    }
    Ok(())
}
