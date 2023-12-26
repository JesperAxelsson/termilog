use std::io;
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

use crate::log_line::LogLine2;
use crate::log_line::LogData;
use crate::ui;

// struct App<'a> {
pub struct App {
    list_items: StatefulList,
    // items: StatefulList<(&'a str, usize)>,
    // events: Vec<(&'a str, &'a str)>,
    show_popup: bool,
}

// impl<'a> App<'a> {
impl App {
    pub fn new(log_data: LogData) -> App {
        App {
            list_items: StatefulList::with_items(log_data),
            show_popup: false,
        }
    }

    // Use this to read file?
    //    fn on_tick(&mut self) {
    //     // let event = self.events.remove(0);
    //     // self.events.push(event);
    // }



    pub fn run_app<B: Backend>(mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

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

            // TODO: Handle notify events here
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
                let mut lines = vec![Line::from(&*i.info())];
                // for _ in 0..i.1 {
                lines.push(Line::from(Span::styled(
                    i.slug(),
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
    fn with_items(items: LogData) -> StatefulList {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.borrow_dependent().0.len() - 1 {
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
                    self.items.borrow_dependent().0.len() - 1
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

    fn selected_item(&mut self) -> Option<&LogLine2> {
        let ix = self.state.selected();
        if let Some(ix) = ix {
            return Some(&self.items.borrow_dependent().0[ix]);
        }

        return None;
    }
}


