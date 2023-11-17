mod app;
mod calpager;
mod help;
mod moon;
use crate::app::App;
use crate::calpager::CalPager;
use crate::moon::Phoon;
use chrono::Local;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;

fn main() -> io::Result<()> {
    let today = Local::now().date_naive();
    let mut terminal = init_terminal()?;
    terminal.hide_cursor()?;

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        let _ = reset_terminal();
        original_hook(panic);
    }));

    let calpager = CalPager::new(today, Phoon);
    let r = App::new(terminal, calpager).run();
    reset_terminal()?;
    r
}

fn init_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    let mut stream = io::stdout();
    execute!(stream, EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stream))
}

fn reset_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
