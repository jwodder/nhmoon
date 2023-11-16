use super::DateStyler;
use chrono::{naive::NaiveDate, Datelike, Month, Weekday, Weekday::*};
use num_traits::cast::FromPrimitive;
use ratatui::{style::Style, text::Span};
use std::collections::VecDeque;
use std::ops::Index;

pub(super) trait WeekdayExt {
    fn index0(&self) -> u16;
    fn index1(&self) -> u16;
}

impl WeekdayExt for Weekday {
    fn index0(&self) -> u16 {
        self.num_days_from_sunday()
            .try_into()
            .expect("number of days from Sunday should fit in a u16")
    }

    fn index1(&self) -> u16 {
        self.number_from_sunday()
            .try_into()
            .expect("number of days from Sunday should fit in a usize")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct StyledDate {
    pub(crate) date: NaiveDate,
    pub(crate) style: Style,
}

impl StyledDate {
    pub(super) fn year(&self) -> i32 {
        self.date.year()
    }

    pub(super) fn month(&self) -> Month {
        Month::from_u32(self.date.month())
            .expect("converting a month number to a Month should not fail")
    }

    pub(super) fn day(&self) -> u32 {
        self.date.day()
    }

    pub(super) fn is_last_day_of_month(&self) -> bool {
        match self.date.succ_opt() {
            Some(tomorrow) => self.date.month() != tomorrow.month(),
            None => true,
        }
    }

    pub(super) fn in_first_week_of_month(&self) -> bool {
        self.date.day0() <= self.date.weekday().index1().into()
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
            next_weekday: Some(Sun),
        }
    }
}

impl<'a> Iterator for EnumerateWeek<'a> {
    type Item = (Weekday, StyledDate);

    fn next(&mut self) -> Option<(Weekday, StyledDate)> {
        if let Some(wd) = self.next_weekday {
            self.next_weekday = match wd.succ() {
                Sun => None,
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

    pub(super) fn around_date(&self, date: NaiveDate, week_qty: usize) -> VecDeque<Week> {
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

    fn make(&self, sunday: NaiveDate) -> Week {
        // TODO: Replace these unwrap()'s with something that returns Result!
        let monday = sunday.succ_opt().unwrap();
        let tuesday = monday.succ_opt().unwrap();
        let wednesday = tuesday.succ_opt().unwrap();
        let thursday = wednesday.succ_opt().unwrap();
        let friday = thursday.succ_opt().unwrap();
        let saturday = friday.succ_opt().unwrap();
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

    fn containing(&self, date: NaiveDate) -> Week {
        let sunday = n_days_before(date, date.weekday().index0().into());
        self.make(sunday)
    }

    pub(super) fn week_before(&self, week: &Week) -> Week {
        self.make(n_days_before(week[Sun].date, 7))
    }

    pub(super) fn week_after(&self, week: &Week) -> Week {
        self.make(n_days_after(week[Sun].date, 7))
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

fn n_days_after(mut date: NaiveDate, n: usize) -> NaiveDate {
    for _ in 0..n {
        date = date.succ_opt().expect("Reached end of calendar");
    }
    date
}

fn n_days_before(mut date: NaiveDate, n: usize) -> NaiveDate {
    for _ in 0..n {
        date = date.pred_opt().expect("Reached beginning of calendar");
    }
    date
}
