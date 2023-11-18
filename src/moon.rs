use crate::calendar::DateStyler;
use ratatui::style::{Style, Stylize};
use time::Date;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
enum Phase {
    Normal,
    Full,
    New,
}

impl Phase {
    fn for_date(date: Date) -> Phase {
        // Will give wrong results pre-1900
        let year = date.year().abs_diff(1900);
        let goldn = (year % 19) + 1;
        let mut epact = (11 * goldn + 18) % 30;
        if (epact == 25 && goldn > 11) || epact == 24 {
            epact += 1;
        }
        match (((((u32::from(date.ordinal()) - 1 + epact) * 6) + 11) % 177) / 22) & 7 {
            0 => Phase::New,
            4 => Phase::Full,
            _ => Phase::Normal,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Phoon;

impl DateStyler for Phoon {
    fn date_style(&self, date: Date) -> Style {
        match Phase::for_date(date) {
            Phase::Normal => Style::new(),
            Phase::Full => Style::new().light_yellow().bold(),
            Phase::New => Style::new().light_blue(),
        }
    }
}
