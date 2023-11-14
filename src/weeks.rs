use chrono::{naive::NaiveDate, Datelike, Month, Weekday, Weekday::*};
use num_traits::cast::FromPrimitive;
use ratatui::{style::Style, text::Span};
use std::collections::VecDeque;
use std::ops::Index;

pub(crate) trait DateStyler {
    fn date_style(&self, date: NaiveDate) -> Style;
}

impl<T: DateStyler + ?Sized> DateStyler for &T {
    fn date_style(&self, date: NaiveDate) -> Style {
        (**self).date_style(date)
    }
}

pub(crate) trait WeekdayExt {
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
pub(crate) struct StyledDate {
    pub(crate) date: NaiveDate,
    pub(crate) style: Style,
}

impl StyledDate {
    pub(crate) fn year(&self) -> i32 {
        self.date.year()
    }

    pub(crate) fn month(&self) -> Month {
        Month::from_u32(self.date.month())
            .expect("converting a month number to a Month should not fail")
    }

    pub(crate) fn day(&self) -> u32 {
        self.date.day()
    }

    pub(crate) fn is_last_day_of_month(&self) -> bool {
        match self.date.succ_opt() {
            Some(tomorrow) => self.date.month() != tomorrow.month(),
            None => true,
        }
    }

    pub(crate) fn in_first_week_of_month(&self) -> bool {
        (self.date.day() as usize) <= self.date.weekday().index1()
    }

    pub(crate) fn show(&self, is_today: bool) -> Span<'static> {
        let s = if is_today {
            format!("[{:2}]", self.day())
        } else {
            format!(" {:2} ", self.day())
        };
        Span::styled(s, self.style)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Week([StyledDate; 7]);

impl Week {
    fn new<S: DateStyler>(sunday: NaiveDate, styler: S) -> Self {
        Week(std::array::from_fn(move |i| {
            let date = n_days_after(sunday, i);
            let style = styler.date_style(date);
            StyledDate { date, style }
        }))
    }
}

impl Index<Weekday> for Week {
    type Output = StyledDate;

    fn index(&self, wd: Weekday) -> &StyledDate {
        &self.0[wd.index0()]
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) struct WeekFactory<S>(S);

impl<S: DateStyler> WeekFactory<S> {
    pub(crate) fn new(styler: S) -> Self {
        WeekFactory(styler)
    }

    pub(crate) fn around_date(&self, date: NaiveDate, week_qty: usize) -> VecDeque<Week> {
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
        Week::new(sunday, &self.0)
    }

    fn containing(&self, date: NaiveDate) -> Week {
        let sunday = n_days_before(date, date.weekday().index0());
        self.make(sunday)
    }

    pub(crate) fn week_before(&self, week: &Week) -> Week {
        self.make(n_days_before(week[Sun].date, 7))
    }

    pub(crate) fn week_after(&self, week: &Week) -> Week {
        self.make(n_days_after(week[Sun].date, 7))
    }

    pub(crate) fn weeks_before(&self, mut week: Week, qty: usize) -> VecDeque<Week> {
        let mut weeks = VecDeque::with_capacity(qty + 1);
        for _ in 0..qty {
            week = self.week_before(&week);
            weeks.push_front(week);
        }
        weeks
    }

    pub(crate) fn weeks_after(&self, mut week: Week, qty: usize) -> VecDeque<Week> {
        let mut weeks = VecDeque::with_capacity(qty + 1);
        for _ in 0..qty {
            week = self.week_after(&week);
            weeks.push_back(week);
        }
        weeks
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WeekdayIter(Option<Weekday>);

impl WeekdayIter {
    pub(crate) fn new() -> Self {
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
