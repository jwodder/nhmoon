use super::DateStyler;
use ratatui::{style::Style, text::Span};
use std::collections::VecDeque;
use std::ops::Index;
use time::{Date, Duration, Month, Weekday, Weekday::*};

pub(super) trait WeekdayExt {
    fn index0(&self) -> u16;
    fn index1(&self) -> u16;
}

impl WeekdayExt for Weekday {
    fn index0(&self) -> u16 {
        self.number_days_from_sunday().into()
    }

    fn index1(&self) -> u16 {
        self.number_from_sunday().into()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct StyledDate {
    pub(crate) date: Date,
    pub(crate) style: Style,
}

impl StyledDate {
    pub(super) fn year(&self) -> i32 {
        self.date.year()
    }

    pub(super) fn month(&self) -> Month {
        self.date.month()
    }

    pub(super) fn day(&self) -> u8 {
        self.date.day()
    }

    pub(super) fn is_last_day_of_month(&self) -> bool {
        match self.date.next_day() {
            Some(tomorrow) => self.date.month() != tomorrow.month(),
            None => true,
        }
    }

    pub(super) fn in_first_week_of_month(&self) -> bool {
        in_first_week_of_month(self.date)
    }

    pub(super) fn show(&self, is_today: bool) -> Span<'static> {
        let s = if is_today {
            format!("[{:2}]", self.day())
        } else {
            format!(" {:2} ", self.day())
        };
        Span::styled(s, self.style)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Week([StyledDate; 7]);

impl Week {
    pub(super) fn enumerate(&self) -> EnumerateWeek<'_> {
        EnumerateWeek::new(self)
    }
}

impl Index<Weekday> for Week {
    type Output = StyledDate;

    fn index(&self, wd: Weekday) -> &StyledDate {
        &self.0[usize::from(wd.index0())]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct EnumerateWeek<'a> {
    week: &'a Week,
    next_weekday: Option<Weekday>,
}

impl<'a> EnumerateWeek<'a> {
    fn new(week: &'a Week) -> Self {
        EnumerateWeek {
            week,
            next_weekday: Some(Sunday),
        }
    }
}

impl<'a> Iterator for EnumerateWeek<'a> {
    type Item = (Weekday, StyledDate);

    fn next(&mut self) -> Option<(Weekday, StyledDate)> {
        if let Some(wd) = self.next_weekday {
            self.next_weekday = match wd.next() {
                Sunday => None,
                wd2 => Some(wd2),
            };
            Some((wd, self.week[wd]))
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(super) struct WeekFactory<S>(S);

impl<S: DateStyler> WeekFactory<S> {
    pub(super) fn new(styler: S) -> Self {
        WeekFactory(styler)
    }

    pub(super) fn around_date(&self, date: Date, week_qty: usize) -> VecDeque<Week> {
        let mut weeks = VecDeque::with_capacity(week_qty + 1);
        let start_week = self.containing(date);
        weeks.push_front(start_week);
        let mut w = start_week;
        for _ in 0..((week_qty - 1) / 2) {
            w = self.week_before(&w);
            weeks.push_front(w);
        }
        w = start_week;
        for _ in 0..(week_qty / 2) {
            w = self.week_after(&w);
            weeks.push_back(w);
        }
        weeks
    }

    fn make(&self, sunday: Date) -> Week {
        // TODO: Replace these unwrap()'s with something that returns Result!
        let monday = sunday.next_day().unwrap();
        let tuesday = monday.next_day().unwrap();
        let wednesday = tuesday.next_day().unwrap();
        let thursday = wednesday.next_day().unwrap();
        let friday = thursday.next_day().unwrap();
        let saturday = friday.next_day().unwrap();
        Week([
            StyledDate {
                date: sunday,
                style: self.0.date_style(sunday),
            },
            StyledDate {
                date: monday,
                style: self.0.date_style(monday),
            },
            StyledDate {
                date: tuesday,
                style: self.0.date_style(tuesday),
            },
            StyledDate {
                date: wednesday,
                style: self.0.date_style(wednesday),
            },
            StyledDate {
                date: thursday,
                style: self.0.date_style(thursday),
            },
            StyledDate {
                date: friday,
                style: self.0.date_style(friday),
            },
            StyledDate {
                date: saturday,
                style: self.0.date_style(saturday),
            },
        ])
    }

    fn containing(&self, date: Date) -> Week {
        let sunday = n_days_before(date, date.weekday().index0().into());
        self.make(sunday)
    }

    pub(super) fn week_before(&self, week: &Week) -> Week {
        self.make(n_days_before(week[Sunday].date, 7))
    }

    pub(super) fn week_after(&self, week: &Week) -> Week {
        self.make(n_days_after(week[Sunday].date, 7))
    }

    pub(super) fn weeks_before(&self, mut week: Week, qty: usize) -> VecDeque<Week> {
        let mut weeks = VecDeque::with_capacity(qty + 1);
        for _ in 0..qty {
            week = self.week_before(&week);
            weeks.push_front(week);
        }
        weeks
    }

    pub(super) fn weeks_after(&self, mut week: Week, qty: usize) -> VecDeque<Week> {
        let mut weeks = VecDeque::with_capacity(qty + 1);
        for _ in 0..qty {
            week = self.week_after(&week);
            weeks.push_back(week);
        }
        weeks
    }
}

fn n_days_after(date: Date, n: i64) -> Date {
    date.checked_add(Duration::days(n))
        .expect("Reached end of calendar")
}

fn n_days_before(date: Date, n: i64) -> Date {
    date.checked_sub(Duration::days(n))
        .expect("Reached beginning of calendar")
}

fn in_first_week_of_month(date: Date) -> bool {
    date.day() <= date.weekday().number_from_sunday()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use time::Month::*;

    #[rstest]
    #[case(2023, October, 1, true)]
    #[case(2023, October, 7, true)]
    #[case(2023, October, 8, false)]
    #[case(2023, November, 1, true)]
    #[case(2023, November, 4, true)]
    #[case(2023, November, 5, false)]
    fn test_in_first_week_of_month(
        #[case] year: i32,
        #[case] month: Month,
        #[case] day: u8,
        #[case] r: bool,
    ) {
        let date = Date::from_calendar_date(year, month, day).unwrap();
        assert_eq!(in_first_week_of_month(date), r);
    }
}
