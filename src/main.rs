mod calpager;
use crate::calpager::calendar_pager;
use chrono::{naive::NaiveDate, Datelike};
use crossterm::style::{ContentStyle, Stylize};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use std::io::stdout;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
enum Phase {
    Normal,
    Full,
    New,
}

impl Phase {
    fn for_date(date: NaiveDate) -> Phase {
        // Will give wrong results pre-1900
        let year = date.year().abs_diff(1900);
        let goldn = (year % 19) + 1;
        let mut epact = (11 * goldn + 18) % 30;
        if (epact == 25 && goldn > 11) || epact == 24 {
            epact += 1;
        }
        match (((((date.ordinal0() + epact) * 6) + 11) % 177) / 22) & 7 {
            0 => Phase::New,
            4 => Phase::Full,
            _ => Phase::Normal,
        }
    }

    fn style(&self) -> ContentStyle {
        match self {
            Phase::Normal => ContentStyle::new(),
            Phase::Full => ContentStyle::new().yellow().on_black(),
            Phase::New => ContentStyle::new().blue().on_black(),
        }
    }
}

fn main() -> crossterm::Result<()> {
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    enable_raw_mode()?; // "Raw mode" here disables echo, right?
                        // TODO: Call SetColors to set fg & bg colors?
    calendar_pager(&mut stdout, &phoon)?;
    // TODO: Ensure this is run on Err:
    disable_raw_mode()?;
    stdout.execute(LeaveAlternateScreen)?;
    Ok(())
}

fn phoon(date: NaiveDate) -> ContentStyle {
    Phase::for_date(date).style()
}
