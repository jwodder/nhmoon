mod util;
mod weeks;
mod widget;
pub(crate) use self::weeks::WeekWindow;
pub(crate) use self::widget::Calendar;
use ratatui::style::Style;
use time::Date;

pub(crate) trait DateStyler {
    fn date_style(&self, date: Date) -> Style;
}
