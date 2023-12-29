use std::{cmp, mem};
use std::io::{Seek, SeekFrom};
use std::{io, time::Duration, fs::metadata};
use std::{
    fs::File,
    io::Read,
};
use log::trace;
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};

use crossterm::event::{Event, KeyCode};
use crossterm::event;

use crate::log_line::LogLine;
use crate::log_line::LogData;
use crate::ui;

#[derive(Debug)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
}

// struct App<'a> {
pub struct App {
    file: FileInfo,
    list_items: StatefulList,
    // items: StatefulList<(&'a str, usize)>,
    // events: Vec<(&'a str, &'a str)>,
    show_popup: bool,
}

// impl<'a> App<'a> {
impl App {
    pub fn new(file: FileInfo, log_data: LogData) -> App {
        App {
            file,
            list_items: StatefulList::with_items(log_data),
            show_popup: false,
        }
    }

    pub fn run_app<B: Backend>(mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if event::poll(Duration::from_millis(1000))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('p') => self.show_popup = !self.show_popup,
                        KeyCode::Left => self.list_items.unselect(),
                        KeyCode::Down => self.list_items.next(),
                        KeyCode::Up => self.list_items.previous(),
                        _ => {}
                    }
                }
            }

            // TODO: Handle notify events here
            if let Ok(meta) = metadata(&self.file.name) {
                if self.file.size != meta.len() {

                    let mut file = File::open(&self.file.name)?;

                    if self.file.size < meta.len() {
                        file.seek(SeekFrom::Start(self.file.size)).unwrap();
                    }

                    let mut contents = String::new();
                    file.read_to_string(&mut contents)?;


                    if self.file.size < meta.len() {
                        trace!("File size increased {:?}", contents);
                        self.list_items.append_text(&contents);
                    } else {
                        trace!("File size reduced {:?} to {:?}", self.file.size, meta.len());
                        let ll = LogData::from_content(contents);
                        self.list_items.change_log_data(ll);
                    }


                    // let parser = raw_parse::RawParser {};
                    // let log_lines = parser.parse_lines(&contents);
                    //
                    // let ll = parser.map_log(contents.clone(), log_lines.clone());

                    // self.list_items.change_log_data(ll);


                    self.file.size = meta.len();

                } else {
                    trace!("No file changed");
                }
            } else {
                trace!("File gone!");
            }

            trace!("Loop!");
        }
    }

     fn ui(&mut self, f: &mut Frame) {
        let size = f.size();

        // let chunks = Layout::default()
        //     .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        //     .split(size);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(f.size());

        // Iterate through all elements in the `items` app and append some debug text to it.
        let items: Vec<ListItem> = self
            .list_items
            .items.borrow_dependent().0
            
            .iter()
            .map(|i| {
                let mut lines = vec![Line::from(i.info())];
                // for _ in 0..i.1 {
                lines.push(Line::from(Span::styled(
                    i.slug(30),
                    Style::default().add_modifier(Modifier::ITALIC),
                )));
                // }
                ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::White))
            })
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let items = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("List"))
            .highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        // We can now render the item list
        f.render_stateful_widget(items, chunks[0], &mut self.list_items.state);

        let text = if self.show_popup {
            "Press p to close the popup"
        } else {
            "Press p to show the popup"
        };
        let paragraph = Paragraph::new(Span::styled(
            text,
            Style::default().add_modifier(Modifier::SLOW_BLINK),
        ))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

        f.render_widget(paragraph, chunks[0]);

        let block = Block::default()
            .title("Content")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Blue));

        // f.render_widget(block, chunks[1]);
        // let (log_text, _) = app.items.selected_item().unwrap_or(&("default", 0));
        if let Some(log_text) = self.list_items.selected_item() {
            let paragraph = Paragraph::new(Span::styled(log_text.text(), Style::default()))
                .block(block)
                // .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            f.render_widget(paragraph, chunks[1]);
        }

        // Render popup
        if self.show_popup {
            let block = Block::default().title("Popup").borders(Borders::ALL);
            let area = ui::centered_rect(60, 20, size);
            f.render_widget(Clear, area); //this clears out the background
            f.render_widget(block, area);
        }
    }


}

// use std::ops::Index;
struct StatefulList {
    state: ListState,
    items: LogData,
}

impl StatefulList {

    pub fn with_items(items: LogData) -> StatefulList {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn change_log_data(&mut self, log_data: LogData) {
        let data_len = log_data.len();
        self.items = log_data;
        if let Some(state) = self.state.selected() {
            *self.state.selected_mut() = Some(cmp::min(state, data_len));
        }
    }

    pub fn append_text(&mut self, content: &str) {
        let items = mem::replace(&mut self.items, LogData::empty());
        self.items = items.append_text(content);
    }

    fn next(&mut self) {
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

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }

    fn selected_item(&mut self) -> Option<&LogLine> {
        let ix = self.state.selected();
        if let Some(ix) = ix {
            return Some(&self.items.log_lines()[ix]);
        }

        None
    }
}


