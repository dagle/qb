use qb::{rows::DbTable, input::{Input, InputType}};
use rusqlite::Connection;
use rusqlite::types::Value;
use clap::Parser;
use std::{path::PathBuf, time::Duration, io::Stdout, convert::TryInto};
use anyhow::{Context, Result, bail};

use std::io;

mod ui;
mod config;

use config::*;

use ratatui::{
    layout::{Constraint, Layout, Direction},
    backend::CrosstermBackend,
    Terminal, prelude::Backend
};

use crossterm::{event::{self, Event, KeyCode}, terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode, LeaveAlternateScreen}, execute};

pub struct Qb {
    conn: Connection,
    pub titles: Vec<String>,
    tables: Vec<Option<DbTable>>,
    pub index: usize,
    mode: Mode,
    // input: Option<Input>,
}

impl Qb {
    pub fn new(conn: Connection) -> Result<Qb> {
        let tbls = {
            let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type ='table' AND name NOT LIKE 'sqlite_%';")?;
            let rows = stmt.query_map([], |row| row.get(0))?;

            let mut tbls = Vec::new();
            for tbl in rows {
                let t: String = tbl?;
                tbls.push(t.clone());
            }
            tbls
        };
        let tables = vec![None; tbls.len()];

        Ok(Qb {
            conn,
            titles: tbls,
            tables,
            index: 0,
            mode: Mode::Main,
            // input: None,
        })
    }

    pub fn populate_table(&mut self, index: usize) -> Result<()> {
        let table = &self.titles[index];
        let query = format!("SELECT * FROM {}", table);
        let (scheme, ents) = self.get_entries(&query)?;
        self.tables[index] = Some(DbTable::new(query, scheme, ents));
        Ok(())
    }

    pub fn custom_seach(&mut self, query: &str) -> Result<()> {
        let (scheme, ents) = self.get_entries(query)?;
        self.tables.push(Some(DbTable::new(query.to_owned(), scheme, ents)));
        self.titles.push("custom search".to_owned());
        Ok(())
    }

    pub fn exec(&self, sql: &str) -> Result<()> {
        self.conn.execute(sql, [])?;
        Ok(())
    }

    pub fn selected(&mut self) -> Result<&DbTable> {
        if self.tables[self.index].is_none() {
            self.populate_table(self.index)?;
        }
        // this unwrap is safe
        Ok(self.tables[self.index].as_ref().unwrap())
    }

    pub fn mutselected(&mut self) -> Result<&mut DbTable> {
        if self.tables[self.index].is_none() {
            self.populate_table(self.index)?;
        }
        Ok(self.tables[self.index].as_mut().unwrap())
    }

    // pub fn select_table(&mut self) -> Result<()> {
    //     if self.tables[self.index].is_none() {
    //         return self.populate_table(self.index)
    //     }
    //     Ok(())
    // }

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

    pub fn tab_last(&mut self) {
        self.index = self.titles.len() - 1;
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
    pub fn reload(&mut self) -> Result<()> {
        self.populate_table(self.index)
    }

    fn get_entries(&self, query: &str) -> Result<(Vec<String>,Vec<Vec<Value>>)> {
        let mut stmt = self.conn.prepare(query)?;
        let value = stmt.column_names();
        let scheme: Vec<String> = value.iter().map(|s| s.to_string()).collect();
        let ncols = stmt.column_count();
        let rows = stmt.query_map([], |row| {
            let mut cols = Vec::new();
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
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    db_path: PathBuf,
}

pub fn startup() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let stdout = io::stdout();
    enable_raw_mode()?;
    execute!(std::io::stderr(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).context("Backend failed")
}

fn shutdown(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn parse_command(input: &str) -> Result<(InputType, String)> {
    let mut parts = input.split_whitespace();

    let kind = parts.next();
    let Some(kind) = kind else {
        bail!("Missing kind");
    };
    let kind = kind.try_into()?;

    let args = parts.fold(String::new(), |a, b| a + b + "\n");

    Ok((kind, args))
}


/// Handle an event
/// Returning a true means that we want to break the loop
fn event<B: Backend>(qb: &mut Qb, cfg: &Config, input: &mut Option<Input>, last_err: &mut Option<anyhow::Error>, terminal: &mut Terminal<B>) -> Result<bool> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            match qb.mode {
                Mode::Main => {
                    if let Some(action) = cfg.main.get(&key) {
                        match action {
                            MainAction::Next => {
                                qb.mutselected()?.next();
                            }
                            MainAction::Prev => {
                                qb.mutselected()?.prev();
                            }
                            MainAction::Hnext => {
                                qb.mutselected()?.hnext();
                            }
                            MainAction::Hprev => {
                                qb.mutselected()?.hprev();
                            }
                            MainAction::First => {
                                qb.mutselected()?.first();
                            }
                            MainAction::Last => {
                                qb.mutselected()?.last();
                            }
                            MainAction::Tnext => {
                                qb.next();
                            }
                            MainAction::Tprev => {
                                qb.prev();
                            }
                            MainAction::Reload => {
                                qb.reload()?;
                            }
                            MainAction::Zoom => {
                                qb.mode = Mode::Zoom;
                            }
                            MainAction::InputCurrent(pree) => {
                                let table = qb.selected()?;
                                let inputstr = format!("{} {}", pree, table.search);
                                *input = Some(Input::new(InputType::Query
                                        , inputstr));
                                terminal.show_cursor()?;
                                qb.mode = Mode::Input;
                                *last_err = None;
                            }
                            MainAction::Input(pree) => {
                                *input = Some(Input::new(InputType::Exec, pree.clone()));
                                terminal.show_cursor()?;
                                qb.mode = Mode::Input;
                                *last_err = None;
                            }
                            MainAction::Quit => {
                                return Ok(true);
                            }
                            MainAction::Search => {
                            }
                        }
                    }
                }
                Mode::Visual => {
                }
                Mode::Zoom => {
                    if let Some(action) = cfg.zoom.get(&key) {
                        match action {
                            ZoomAction::Back => {
                                qb.mode = Mode::Main;
                            }
                            ZoomAction::ZoomIn => {
                                let table = qb.mutselected()?;
                                table.zoom.zoom_in();
                            }
                            ZoomAction::ZoomOut => {
                                let table = qb.mutselected()?;
                                table.zoom.zoom_out(table.hlen);
                            }
                            ZoomAction::Next => {
                                let table = qb.mutselected()?;
                                table.zoom.next(table.hlen);
                            }
                            ZoomAction::Prev => {
                                let table = qb.mutselected()?;
                                table.zoom.prev();
                            }
                        }
                    }
                }
                Mode::Input => {
                    if let Some(action) = cfg.input.get(&key) {
                        match action {
                            InputAction::Leave => {
                                qb.mode = Mode::Main;
                            }
                            InputAction::Enter => {
                                if let Some(ref inner) = input { 
                                    let res = parse_command(inner.input.value());
                                    *input = None;
                                    qb.mode = Mode::Main;
                                    terminal.hide_cursor()?;
                                    let (kind, args) = res?;

                                    match kind {
                                        InputType::Exec => {
                                            qb.exec(&args)?;
                                        }
                                        InputType::Query => {
                                            let res = qb.custom_seach(&args);
                                            if res.is_ok() {
                                                qb.tab_last()
                                            }
                                            res?;
                                        }
                                    }
                                }
                            }
                            action => {
                                if let Some(ref mut input) = input { 
                                    let req = action.try_into()?;
                                    input.handle(req);
                                }
                            }
                        }
                    } else if let Some(ref mut input) = input { 
                        if let KeyCode::Char(char) = key.code {
                            input.handle(tui_input::InputRequest::InsertChar(char));
                        }
                    }
                }
            }
        }
    }
    Ok(false)
}

fn run_app<B: Backend>(mut qb: Qb, cfg: Config, terminal: &mut Terminal<B>) -> Result<()> {
    let mut input: Option<Input> = None;
    let mut last_err: Option<anyhow::Error> = None;
    'lp: loop {
        let mode = qb.mode;
        terminal.draw(|f| {
            let rect = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)].as_ref())
                .split(f.size());
            ui::make_tabs(&qb, f, rect[0]);
            let table = qb.mutselected().expect("Couldn't select table");
            table.render(f, rect[1]);
            match mode {
                Mode::Main => {
                    if let Some(ref err) = last_err {
                        ui::input_err(&err.to_string(), f, rect[2])
                    }
                }
                Mode::Visual => {
                    // higligt the selected fields in grey?
                }
                Mode::Zoom => {
                    table.zoom.render(table, f)
                }
                Mode::Input => {
                    if let Some(ref input) = input {
                        input.render(f, rect[2])
                    }
                }
            }
            // ui::make_tabs(&qb, f, rect[2])
        }).context("render error")?;
        match event(&mut qb, &cfg, &mut input, &mut last_err, terminal) {
            Ok(false) => {}, //
            Ok(true) => {
                break 'lp;
            }
            Err(err) => {
                last_err = Some(err)
            }
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let cfg: Config = confy::load("qb", None)
        .context("Couldn't load config file, remove it to get a new one")?;
    confy::store("qb", None, &cfg).context("Couldn't update config")?;

    let path = &args.db_path;
    let conn = Connection::open(path).context("Failed to connect to db")?;
    let qb = Qb::new(conn)?;
 
    let mut terminal = startup()?;

    terminal.clear().context("Clear error")?;

    let res = run_app(qb, cfg, &mut terminal);
   
    // move draw.
    shutdown(&mut terminal)?;
    res?;
    Ok(())
}
