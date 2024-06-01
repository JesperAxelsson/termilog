use log::trace;
use ratatui::{prelude::*, widgets::*};
use std::io::{Seek, SeekFrom};
use std::{fs::metadata, io, time::Duration};
use std::{fs::File, io::Read};

use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};

use tui_textarea::TextArea;

use crate::log_line::LogData;
use crate::stateful_list::StatefulList;
use crate::ui;

#[derive(Debug)]
pub struct KeyBinding {
    pub key: KeyCode,
    pub description: String,
    pub command: Command,
}

impl KeyBinding {
    pub fn new(key: KeyCode, description: String, command: Command) -> Self {
        Self {
            key,
            description,
            command,
        }
    }
}

#[derive(Debug)]
pub enum Command {
    Quit,
    ShowKeybindings,
    Filter,
    Follow,
    ListUp,
    ListDown,
    Unselect,
}

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

    filter: Option<Vec<String>>,

    input_mode: InputMode,

    keybindings_selected: Option<usize>,
    keybindings_length: usize,
    keybindings: Vec<KeyBinding>,
    keybindings_state: TableState,
    show_keybindings: bool,

    show_filter: bool,

    textarea: TextArea<'a>,

    exit: bool,
}

enum InputMode {
    Normal,
    EditingFilter,
    ShowingKeybindings,
}

// pub enum AppState {
//     Browsing,
//     WaitingForFile,
//     ShowKeybindings,
//     ShowFilter,
// }

// impl<'a> App<'a> {
impl<'a> App<'a> {
    pub fn new(file: FileInfo, log_data: LogData) -> App<'a> {
        let mut textarea = TextArea::default();
        textarea.set_block(Block::default().title("Filter").borders(Borders::ALL));

        let keybindings = vec![
            KeyBinding::new(
                KeyCode::Char('f'),
                "Toggle follow new logs".to_owned(),
                Command::Follow,
            ),
            KeyBinding::new(
                KeyCode::Char('/'),
                "Filter on things".to_owned(),
                Command::Filter,
            ),
            KeyBinding::new(
                KeyCode::Char('q'),
                "Exit application".to_owned(),
                Command::Quit,
            ),
            KeyBinding::new(KeyCode::Up, "Move list up".to_owned(), Command::ListUp),
            KeyBinding::new(
                KeyCode::Down,
                "Move list down".to_owned(),
                Command::ListDown,
            ),
        ];

        App {
            file,
            list_items: StatefulList::with_items(log_data),

            follow_mode: false,

            filter: None,

            input_mode: InputMode::Normal,

            keybindings,
            keybindings_selected: None,
            keybindings_length: 50,
            keybindings_state: TableState::default(),
            show_keybindings: false,

            show_filter: false,

            textarea,

            exit: false,
        }
    }

    pub fn run_app<B: Backend>(mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|f| self.ui(f))?;

            self.handle_events()?;

            self.listen_file_notification()?;

            // trace!("Loop!");
        }

        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    self.exit = true;
                    return Ok(());
                }

                match self.input_mode {
                    InputMode::Normal => {
                        match key.code {
                            // TODO: Handle page up/down and Home/End
                            KeyCode::Char('f') => self.follow_mode = !self.follow_mode,
                            KeyCode::Char('?') => {
                                self.show_keybindings = true;
                                self.input_mode = InputMode::ShowingKeybindings;
                            }
                            KeyCode::Char('/') => {
                                self.input_mode = InputMode::EditingFilter;
                                self.show_filter = true;
                            }
                            KeyCode::Left => self.list_items.unselect(),
                            KeyCode::Down => self.list_items.next(),
                            KeyCode::Up => self.list_items.previous(),
                            KeyCode::Esc => {
                                self.hide_popups();
                            }
                            _ => {}
                        }
                    }
                    InputMode::EditingFilter if key.kind == KeyEventKind::Press => match key.code {
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
                        _ => {
                            self.textarea.input(key);
                        }
                    },
                    InputMode::EditingFilter => {} // Key up. We handle all events above
                    InputMode::ShowingKeybindings => {
                        match key.code {
                            KeyCode::Up => {
                                if let Some(n) = self.keybindings_selected {
                                    if n > 0 {
                                        self.keybindings_selected =
                                            Some((n - 1).clamp(0, self.keybindings_length - 1));
                                    } else {
                                        self.keybindings_selected = Some(0);
                                    }
                                } else {
                                    self.keybindings_selected = Some(0);
                                }

                                self.keybindings_state.select(self.keybindings_selected);
                                trace!("Keybind up {:?}", self.keybindings_state);
                            }
                            KeyCode::Down => {
                                if let Some(n) = self.keybindings_selected {
                                    self.keybindings_selected =
                                        Some((n + 1).clamp(0, self.keybindings_length - 1));
                                } else {
                                    self.keybindings_selected = Some(0);
                                }
                                self.keybindings_state.select(self.keybindings_selected);
                                trace!("Keybind down {:?}", self.keybindings_state);
                            }
                            KeyCode::Esc => {
                                trace!("Input: {:?}", self.filter);

                                self.input_mode = InputMode::Normal;
                                self.hide_popups();
                            }
                            // KeyCode::Up => self.list_items.previous(),
                            _ => {}
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn listen_file_notification(&mut self) -> io::Result<()> {
        // Handle notify events here
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
                // trace!("No file changed");
            }
        } else {
            trace!("File gone!");
        }

        Ok(())
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
            .items
            .borrow_dependent()
            .0
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
            // self.keybindings_state = TableState::default();
            self.render_key_bindings(f);
            // let block = Block::default().title("Keybindings").borders(Borders::ALL);
            // let area = ui::centered_rect(60, 20, size);
            // f.render_widget(Clear, area); //this clears out the background
            // f.render_widget(block, area);
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

    fn render_key_bindings(&mut self, f: &mut Frame) {
        let area = ui::centered_rect(60, 20, f.size());

        f.render_widget(Clear, area); //this clears out the background

        let rows = self.keybindings.iter().map(|kb| {
            Row::new(vec![
                format!("{:?}", kb.key).to_owned(),
                kb.description.clone(),
            ])
        });

        let widths = [Constraint::Length(9), Constraint::Percentage(90)];

        let title_block = Block::default().title("Keybindings").borders(Borders::ALL);
        let table = Table::new(rows, widths).block(title_block);

        f.render_stateful_widget(
            table,
            area, //     .inner(&Margin {
            //     vertical: 2,
            //     horizontal: 2,
            // })
            // .offset(Offset { x: 1, y: 1 }),
            &mut self.keybindings_state,
        )
    }
}
