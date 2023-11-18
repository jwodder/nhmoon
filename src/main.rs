mod app;
mod calendar;
mod help;
mod moon;
use crate::app::{App, CrossTerminal};
use crate::calendar::WeekWindow;
use crate::moon::Phoon;
use anyhow::Context;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;
use time::OffsetDateTime;

fn main() -> anyhow::Result<()> {
    let today = OffsetDateTime::now_local()
        .context("failed to determine local date")?
        .date();
    with_terminal(|mut terminal| {
        terminal.hide_cursor().context("failed to hide cursor")?;
        let calpager = WeekWindow::new(today, Phoon);
        App::new(terminal, calpager).run()?;
        Ok(())
    })
}

fn with_terminal<F, T>(func: F) -> anyhow::Result<T>
where
    F: Fn(CrossTerminal) -> anyhow::Result<T>,
{
    let mut stream = io::stdout();
    execute!(stream, EnterAlternateScreen).context("failed to start alternate screen")?;
    if let Err(e) = enable_raw_mode() {
        let _ = execute!(stream, LeaveAlternateScreen);
        return Err(e).context("failed to enable raw terminal mode");
    }

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        let _ = reset_terminal();
        original_hook(panic);
    }));

    let terminal =
        Terminal::new(CrosstermBackend::new(stream)).context("failed to create Terminal object")?;
    let r = func(terminal);
    reset_terminal().context("failed to reset terminal")?;
    r
}

fn reset_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
