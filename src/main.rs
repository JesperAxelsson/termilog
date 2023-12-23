use std::{
    env,
    error::Error,
    fs::{metadata, File},
    io::{self, Read},
    process::exit,
    time::Instant,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

mod log_line;
mod parse_log;
mod raw_parse;

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
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

    fn selected_item(&mut self) -> Option<&T> {
        let ix = self.state.selected();
        if let Some(ix) = ix {
            return Some(&self.items[ix]);
        }

        return None;
    }
}

// struct App<'a> {
struct App {
    items: StatefulList<log_line::LogLine>,
    // items: StatefulList<(&'a str, usize)>,
    // events: Vec<(&'a str, &'a str)>,
    show_popup: bool,
}

// impl<'a> App<'a> {
impl App {
    // fn new() -> App<'a> {
    //     App {
    //         items: StatefulList::with_items(vec![
    //             ("Item0", 1),
    //             ("Item1", 2),
    //             ("Item2", 1),
    //             ("Item3", 3),
    //             ("Item4", 1),
    //             ("Item5", 4),
    //             ("Item6", 1),
    //         ]),
    //         show_popup: false,
    //     }
    // }

    // fn new(log_lines: Vec<log_line::LogLine>) -> App<'a> {
    fn new(log_lines: Vec<log_line::LogLine>) -> App {
        App {
            items: StatefulList::with_items(log_lines),
            show_popup: false,
        }
    }

    // Use this to read file?
    //    fn on_tick(&mut self) {
    //     // let event = self.events.remove(0);
    //     // self.events.push(event);
    // }
}

fn main() -> Result<(), Box<dyn Error>> {
    let now = Instant::now();

    let args: Vec<String> = env::args().collect();
    dbg!(&args);
    if args.len() == 1 {
        eprintln!("Missing parameter: log file!");
        exit(-1);
    }

    let log_path = args[1].clone();

    let meta = metadata(&log_path).expect("Failed to get meta data from path");
    if !meta.is_file() {
        eprintln!("Parameter log file is not a file!");
        exit(-1);
    }

    let mut file = File::open(log_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    println!("Read file: {}ms", now.elapsed().as_millis());
    let now = Instant::now();

    let parser = raw_parse::RawParser {};
    let log_lines = parser.parse_lines(&contents);

    let ll = parser.map_log(contents.clone(), log_lines.clone());
    for l in ll.borrow_dependent().0.iter() {
        println!("Lines: {} ",  l);
    }
        

    println!(
        "Number of lines: {} in {}ms",
        log_lines.len(),
        now.elapsed().as_millis()
    );

return Ok(());
    let now = Instant::now();

    let parser = parse_log::Parser {};
    let log_lines = parser.parse_lines(&contents);

    println!(
        "Number of lines: {} in {}ms",
        log_lines.len(),
        now.elapsed().as_millis()
    );


return Ok(());
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new(log_lines);
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('p') => app.show_popup = !app.show_popup,
                KeyCode::Left => app.items.unselect(),
                KeyCode::Down => app.items.next(),
                KeyCode::Up => app.items.previous(),
                _ => {}
            }
        }

        // TODO: Handle notify events here
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();

    // let chunks = Layout::default()
    //     .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
    //     .split(size);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    // Iterate through all elements in the `items` app and append some debug text to it.
    let items: Vec<ListItem> = app
        .items
        .items
        .iter()
        .map(|i| {
            let mut lines = vec![Spans::from(&*i.date)];
            // for _ in 0..i.1 {
            lines.push(Spans::from(Span::styled(
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
    f.render_stateful_widget(items, chunks[0], &mut app.items.state);

    let text = if app.show_popup {
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
    if let Some(log_text) = app.items.selected_item() {
        let paragraph = Paragraph::new(Span::styled(&log_text.text, Style::default()))
            .block(block)
            // .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, chunks[1]);
    }

    // Render popup
    if app.show_popup {
        let block = Block::default().title("Popup").borders(Borders::ALL);
        let area = centered_rect(60, 20, size);
        f.render_widget(Clear, area); //this clears out the background
        f.render_widget(block, area);
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
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
