use log::info;
use log::LevelFilter;
use std::{
    env,
    error::Error,
    fs::{metadata, File},
    io::{self, Read},
    process::exit,
    time::Instant,
};
use ratatui::Terminal;
use ratatui::prelude::CrosstermBackend;

use crossterm::{
    // event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use app_data::App;

use crate::app_data::FileInfo;


mod log_line;
// mod parse_log;
mod raw_parse;
mod ui;
mod app_data;


fn main() -> Result<(), Box<dyn Error>> {
    // let _ = simple_logging::log_to_file("test.log", LevelFilter::Error);
    let _ = simple_logging::log_to_file("test.log", LevelFilter::Trace);
    info!("Starting up!");

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

    let file_size = meta.len();

    let mut file = File::open(&log_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    println!("Read file: {}ms", now.elapsed().as_millis());
    let now = Instant::now();

    let parser = raw_parse::RawParser {};
    let log_lines = parser.parse_lines(&contents);

    let ll = parser.map_log(contents.clone(), log_lines.clone());

    println!(
        "Number of lines: {} in {}ms",
        log_lines.len(),
        now.elapsed().as_millis()
    );
   
    // setup terminal
    enable_raw_mode()?;
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    execute!(stdout, EnterAlternateScreen
    // , EnableMouseCapture
)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new(
        FileInfo {
            name: log_path,
            size: file_size,
        },
        ll);
    let res = app.run_app(&mut terminal);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        // DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

