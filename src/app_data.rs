use std::io::{Seek, SeekFrom};
use std::{io, time::Duration, fs::metadata};
use std::{
    fs::File,
    io::Read,
};
use log::trace;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};

use crossterm::event::{Event, KeyCode, KeyEventKind};
use crossterm::event;

use tui_textarea::TextArea;

use crate::log_line::LogData;
use crate::stateful_list::StatefulList;
use crate::ui;

#[derive(Debug)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
}

// struct App<'a> {
pub struct App<'a> {
    file: FileInfo,
    list_items: StatefulList,

    follow_mode: bool,

    filter : Option<Vec<String>>,

    input_mode: InputMode,
    
    show_keybindings: bool,
    show_filter: bool,

    textarea: TextArea<'a>,
}

enum InputMode {
    Normal,
    Editing,
}

// impl<'a> App<'a> {
impl<'a> App<'a> {
    pub fn new(file: FileInfo, log_data: LogData) -> App<'a> {
        let mut textarea = TextArea::default();
        textarea.set_block(Block::default().title("Filter").borders(Borders::ALL));


        App {
            file,
            list_items: StatefulList::with_items(log_data),
            follow_mode: false,

            filter: None,

            input_mode: InputMode::Normal,

            show_keybindings: false,
            show_filter: false,

            textarea,
        }
    }

    pub fn run_app<B: Backend>(mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
                loop {
            terminal.draw(|f| self.ui(f))?;

            if event::poll(Duration::from_millis(200))? {
                if let Event::Key(key) = event::read()? {
                    match self.input_mode  {
                        InputMode::Normal => {
                            match key.code {
                                // TODO: Handle page up/down and Home/End
                                KeyCode::Char('q') => return Ok(()),
                                KeyCode::Char('f') => self.follow_mode = !self.follow_mode,
                                KeyCode::Char('?') => self.show_keybindings = !self.show_keybindings,
                                KeyCode::Char('/') => {
                                    self.input_mode = InputMode::Editing;
                                    self.show_filter = true;
                                },
                                KeyCode::Left => self.list_items.unselect(),
                                KeyCode::Down => self.list_items.next(),
                                KeyCode::Up => self.list_items.previous(),
                                KeyCode::Esc => {
                                    self.hide_popups();
                                }
                                _ => {}
                            }
                        },
                        InputMode::Editing if key.kind == KeyEventKind::Press => {
                            match key.code {
                                KeyCode::Enter => {
                                    self.filter = Some(self.textarea.lines().iter().cloned().collect());
                                    trace!("Filter: {:?}", self.filter);

                                    self.input_mode = InputMode::Normal;
                                    self.hide_popups()
                                }
                                KeyCode::Esc => {
                                    trace!("Input: {:?}", self.filter);

                                    self.input_mode = InputMode::Normal;
                                    self.hide_popups();
                                }
                                _ => { self.textarea.input(key); },
                            }
                        }
                        _ => { }
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

                    self.file.size = meta.len();

                    if self.follow_mode {
                        self.list_items.goto_end();
                    }

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

        let block = Block::default()
            .title("Content")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Blue));

        if let Some(log_text) = self.list_items.selected_item() {
            let paragraph = Paragraph::new(Span::styled(log_text.text(), Style::default()))
                .block(block)
                // .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            f.render_widget(paragraph, chunks[1]);
        }

        // Render popup
        if self.show_keybindings {
            let block = Block::default().title("Keybindings").borders(Borders::ALL);
            let area = ui::centered_rect(60, 20, size);
            f.render_widget(Clear, area); //this clears out the background
            f.render_widget(block, area);
        }
        
        if self.show_filter {
            let area = ui::centered_rect(60, 20, size);
            f.render_widget(Clear, area); //this clears out the background
            f.render_widget(self.textarea.widget(), area);
        }

    }

    fn hide_popups(&mut self) {
        self.show_filter = false;
        self.show_keybindings = false;
    }
}


