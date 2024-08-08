use log::trace;
use ratatui::{prelude::*, widgets::*};
use std::io::{Seek, SeekFrom};
use std::{fs::metadata, io, time::Duration};
use std::{fs::File, io::Read};

use crossterm::event::{self, KeyEvent};
use crossterm::event::{Event, KeyCode, KeyEventKind};

use tui_textarea::TextArea;

use crate::log_line::LogData;
use crate::stateful_list::StatefulList;
use crate::ui::{self, make_title};

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
    size: Rect,
    file: FileInfo,
    /// Parsed logs
    list_items: StatefulList,

    /// Should scroll as new logs come in
    follow_mode: bool,

    /// List of regex filters
    filter: Option<Vec<String>>,

    /// Current app state
    app_mode: AppMode,

    /// Show keybindings
    /// TODO: Make keybingings configurable
    keybindings: Vec<KeyBinding>,
    keybindings_state: TableState,

    textarea: TextArea<'a>,
    log_textarea: Option<TextArea<'a>>,

    /// Should exit
    exit: bool,
}

#[derive(Debug, PartialEq, Eq)]
enum AppMode {
    Normal,
    FocusLogText,
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
            size: Rect::default(),
            file,
            list_items: StatefulList::with_items(log_data),

            follow_mode: false,

            filter: None,

            app_mode: AppMode::Normal,

            keybindings,
            keybindings_state: TableState::default(),

            textarea,
            log_textarea: None,

            exit: false,
        }
    }

    pub fn run_app<B: Backend>(mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        while !self.exit {
            if self.follow_mode {
                self.list_items.goto_end();
            }

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
                // Global key commands
                match key.code {
                    KeyCode::Char('q') => {
                        self.exit = true;
                        return Ok(());
                    }
                    _ => {}
                }

                match self.app_mode {
                    AppMode::Normal => self.handle_events_log_list(key)?,
                    AppMode::FocusLogText => self.handle_events_log_text(key)?,
                    AppMode::EditingFilter => self.handle_events_filter(key)?,
                    AppMode::ShowingKeybindings => self.handle_events_show_keybindings(key)?,
                }
            }
        }

        Ok(())
    }

    fn handle_events_log_list(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            KeyCode::Char('f') => self.follow_mode = !self.follow_mode,
            KeyCode::Char('c') => {
                self.list_items.clear_all();
                self.log_textarea = None;
            }
            KeyCode::Char('x') => self.list_items.set_cutoff(0),
            KeyCode::Char('?') => self.app_mode = AppMode::ShowingKeybindings,
            KeyCode::Char('/') => self.app_mode = AppMode::EditingFilter,
            KeyCode::Tab => {
                if self.list_items.selected_item().is_some() {
                    self.app_mode = AppMode::FocusLogText;
                }
            }

            KeyCode::PageUp => self
                .list_items
                .jump_relative(-((self.size.height - 4) as isize)),
            KeyCode::PageDown => self
                .list_items
                .jump_relative((self.size.height - 4) as isize),

            KeyCode::Home => self.list_items.goto_start(),
            KeyCode::End => self.list_items.goto_end(),
            KeyCode::Left => self.list_items.unselect(),
            KeyCode::Down => self.list_items.next(),
            KeyCode::Up => self.list_items.previous(),
            KeyCode::Esc => {
                self.hide_popups();
            }
            _ => {}
        }

        self.update_logtext();

        Ok(())
    }

    fn handle_events_log_text(&mut self, key: KeyEvent) -> io::Result<()> {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Esc => self.app_mode = AppMode::Normal,
                _ => {
                    if let Some(text_area) = &mut self.log_textarea {
                        text_area.input(key);
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_events_filter(&mut self, key: KeyEvent) -> io::Result<()> {
        if key.kind == KeyEventKind::Press {
            match key.code {
                // KeyCode::Enter => {
                //     self.filter = Some(self.textarea.lines().iter().cloned().collect());
                //     trace!("Filter: {:?}", self.filter);
                //
                //     self.app_mode = AppMode::Normal;
                //     self.hide_popups()
                // }
                KeyCode::Esc => {
                    trace!("Input: {:?}", self.filter);

                    self.app_mode = AppMode::Normal;
                    self.hide_popups();
                }
                _ => {
                    self.textarea.input(key);
                }
            }
        }

        Ok(())
    }

    fn handle_events_show_keybindings(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            KeyCode::Up => {
                if let Some(n) = self.keybindings_state.selected_mut() {
                    if *n > 0 {
                        *n = (*n - 1).clamp(0, self.keybindings.len() - 1);
                    } else {
                        *n = 0;
                    }
                } else {
                    self.keybindings_state.select(Some(0));
                }
                // trace!("Keybind up {:?}", self.keybindings_state);
            }
            KeyCode::Down => {
                if let Some(n) = self.keybindings_state.selected_mut() {
                    *n = (*n + 1).clamp(0, self.keybindings.len() - 1);
                } else {
                    self.keybindings_state.select(Some(0));
                }
                // trace!("Keybind down {:?}", self.keybindings_state);
            }
            KeyCode::Esc => {
                self.app_mode = AppMode::Normal;
                self.hide_popups();
            }
            _ => {}
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
                    self.update_logtext();
                }
            } else {
                // trace!("No file changed");
            }
        } else {
            trace!("File gone!");
        }

        Ok(())
    }

    fn update_logtext(&mut self) {
        if let Some(log_text) = self.list_items.selected_item() {
            let ss: String = log_text.text().to_owned();
            let lines: Vec<_> = ss.lines().map(String::from).collect();
            self.log_textarea = Some(TextArea::new(lines));
        } else {
            self.log_textarea = None;
        }
    }

    fn ui(&mut self, f: &mut Frame) {
        let size = f.area();
        self.size = size;

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(f.area());

        self.render_log_list(f, &chunks[0]);
        self.render_full_log(f, &chunks[1]);

        // Render popup
        if self.app_mode == AppMode::ShowingKeybindings {
            self.render_key_bindings(f);
        }

        if self.app_mode == AppMode::EditingFilter {
            let area = ui::centered_rect(60, 20, size);
            f.render_widget(Clear, area); //this clears out the background
            f.render_widget(&self.textarea, area);
        }
    }

    fn render_full_log(&mut self, f: &mut Frame, area: &Rect) {
        let block = Block::default()
            .title(make_title(
                "Content",
                self.app_mode == AppMode::FocusLogText,
            ))
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Blue));

        // if let Some(log_text) = self.list_items.selected_item() {
        if let Some(log_textarea) = &mut self.log_textarea {
            log_textarea.set_block(block);
            f.render_widget(&*log_textarea, *area);
        }
        // }
    }

    fn render_log_list(&mut self, f: &mut Frame, area: &Rect) {
        // Iterate through all elements in the `items` app and append some debug text to it.
        // TODO: Cache or something se we don't recreate this every render
        let mut item_state = self.list_items.state.clone();
        let items: Vec<ListItem> = self
            .list_items
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

        let highlight_symbol = if self.app_mode == AppMode::Normal {
            ">> "
        } else {
            "   "
        };

        // Create a List from all list items and highlight the currently selected one
        let list_widget = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(make_title("List", self.app_mode == AppMode::Normal)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(highlight_symbol);

        // Follow the list here?

        // We can now render the item list
        f.render_stateful_widget(list_widget, *area, &mut item_state);
        self.list_items.state = item_state;
    }

    fn hide_popups(&mut self) {
        self.app_mode = AppMode::Normal;
    }

    fn render_key_bindings(&mut self, f: &mut Frame) {
        let area = ui::centered_rect(60, 20, f.area());

        f.render_widget(Clear, area); //this clears out the background

        let rows = self.keybindings.iter().map(|kb| {
            Row::new(vec![
                format!("{:?}", kb.key).to_owned(),
                kb.description.clone(),
            ])
        });

        let widths = [Constraint::Length(9), Constraint::Percentage(90)];
        // let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);

        let title_block = Block::default().title("Keybindings").borders(Borders::ALL);
        let table = Table::new(rows, widths)
            .block(title_block)
            .highlight_symbol(">");

        f.render_stateful_widget(table, area, &mut self.keybindings_state)
    }
}
