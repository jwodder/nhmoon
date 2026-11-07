mod util;
mod weeks;
mod widget;
pub(crate) use self::weeks::WeekWindow;
pub(crate) use self::widget::Calendar;
use jiff::civil::Date;
use ratatui::style::Style;

pub(crate) trait DateStyler {
    fn date_style(&self, date: Date) -> Style;
}
