#![allow(dead_code)]

use clap::Parser;
use crossterm::{
    // event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::info;
use log::LevelFilter;
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;
use std::{
    error::Error,
    fs::{metadata, File},
    io::{self, Read},
    process::exit,
    time::Instant,
};

use app_data::App;

use crate::app_data::FileInfo;

mod log_line;
mod app_data;
mod raw_parse;
mod stateful_list;
mod ui;


// # Planned and missing features:
// - Show enabled modes and status in status bar
// - Clear toggle to empty visible logs
// - Scroll bar for logs...
// - Fix formatting in the full size log window
// - Group duplicated messages


/// Laravel log reader
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Log file to read from
    #[arg(default_value = "storage/log/laravel.log")]
    log_path: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    // let _ = simple_logging::log_to_file("test.log", LevelFilter::Error);
    let _ = simple_logging::log_to_file("test.log", LevelFilter::Trace);
    info!("Starting up!");
    let args = Args::parse();

    let now = Instant::now();

    let meta = metadata(&args.log_path).expect("Failed to get meta data from path");
    if !meta.is_file() {
        eprintln!("Parameter log file is not a file!");
        exit(-1);
    }

    let file_size = meta.len();

    let mut file = File::open(&args.log_path)?;
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
    execute!(
        stdout,
        EnterAlternateScreen // , EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new(
        FileInfo {
            name: args.log_path,
            size: file_size,
        },
        ll,
    );
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
