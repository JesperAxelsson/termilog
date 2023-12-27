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
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use app_data::App;


mod log_line;
// mod parse_log;
mod raw_parse;
mod ui;
mod app_data;


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
    // for l in ll.borrow_dependent().0.iter() {
    //     println!("Date: {:?} ",  l.date());
    //     println!("Lines: {:?} ",  l.source);
    // }
        

    println!(
        "Number of lines: {} in {}ms",
        log_lines.len(),
        now.elapsed().as_millis()
    );

    // let now = Instant::now();
//
//     let parser = parse_log::Parser {};
//     let log_lines = parser.parse_lines(&contents);
//
//     println!(
//         "Number of lines: {} in {}ms",
//         log_lines.len(),
//         now.elapsed().as_millis()
//     );
//
//
// return Ok(());
    
   
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new(ll);
    let res = app.run_app(&mut terminal);

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

