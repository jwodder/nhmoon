mod app;
mod calendar;
mod help;
mod jumpto;
mod moon;
mod theme;
use crate::app::App;
use crate::calendar::WeekWindow;
use crate::moon::Phoon;
use anyhow::Context;
use lexopt::{Arg, Parser, ValueExt};
use time::{format_description::FormatItem, macros::format_description, Date, OffsetDateTime};

static YMD_FMT: &[FormatItem<'_>] = format_description!("[year]-[month]-[day]");

#[derive(Clone, Debug, Eq, PartialEq)]
enum Command {
    Run { date: Option<Date> },
    Help,
    Version,
}

impl Command {
    fn from_parser(mut parser: Parser) -> Result<Command, lexopt::Error> {
        let mut date = None;
        while let Some(arg) = parser.next()? {
            match arg {
                Arg::Short('h') | Arg::Long("help") => return Ok(Command::Help),
                Arg::Short('V') | Arg::Long("version") => return Ok(Command::Version),
                Arg::Value(value) if date.is_none() => {
                    let value = value.string()?;
                    match Date::parse(&value, &YMD_FMT) {
                        Ok(d) => date = Some(d),
                        Err(e) => {
                            return Err(lexopt::Error::ParsingFailed {
                                value,
                                error: Box::new(e),
                            })
                        }
                    }
                }
                _ => return Err(arg.unexpected()),
            }
        }
        Ok(Command::Run { date })
    }

    fn run(self) -> anyhow::Result<()> {
        match self {
            Command::Run { date } => {
                let today = OffsetDateTime::now_local()
                    .context("failed to determine local date")?
                    .date();
                let mut calpager = WeekWindow::new(today, Phoon);
                if let Some(date) = date {
                    calpager = calpager.start_date(date);
                }
                let terminal = ratatui::init();
                let r = App::new(calpager).run(terminal);
                ratatui::restore();
                r.map_err(Into::into)
            }
            Command::Help => {
                println!("Usage: nhmoon [YYYY-MM-DD]");
                println!();
                println!("Scrollable terminal calendar highlighting NetHack's new & full moons");
                println!();
                println!("Options:");
                println!("  -h, --help        Display this help message and exit");
                println!("  -V, --version     Show the program version and exit");
                Ok(())
            }
            Command::Version => {
                println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
                Ok(())
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    Command::from_parser(Parser::from_env())?.run()
}
