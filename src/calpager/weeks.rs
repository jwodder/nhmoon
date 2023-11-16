use super::DateStyler;
use chrono::{naive::NaiveDate, Datelike, Month, Weekday, Weekday::*};
use num_traits::cast::FromPrimitive;
use ratatui::{style::Style, text::Span};
use std::collections::VecDeque;
use std::ops::Index;

pub(super) trait WeekdayExt {
    fn index0(&self) -> usize;
    fn index1(&self) -> usize;
}

impl WeekdayExt for Weekday {
    fn index0(&self) -> usize {
        self.num_days_from_sunday()
            .try_into()
            .expect("number of days from Sunday should fit in a usize")
    }

    fn index1(&self) -> usize {
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
        (self.date.day() as usize) <= self.date.weekday().index1()
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

impl Index<Weekday> for Week {
    type Output = StyledDate;

    fn index(&self, wd: Weekday) -> &StyledDate {
        &self.0[wd.index0()]
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
        let sunday = n_days_before(date, date.weekday().index0());
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct WeekdayIter(Option<Weekday>);

impl WeekdayIter {
    pub(super) fn new() -> Self {
        WeekdayIter(Some(Sun))
    }
}

impl Iterator for WeekdayIter {
    type Item = Weekday;

    fn next(&mut self) -> Option<Weekday> {
        let r = self.0;
        if let Some(wd) = r {
            let wd = wd.succ();
            if wd == Sun {
                self.0 = None;
            } else {
                self.0 = Some(wd);
            }
        }
        r
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
