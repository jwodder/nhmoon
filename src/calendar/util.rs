use super::DateStyler;
use ratatui::{style::Style, text::Span};
use std::collections::VecDeque;
use std::iter::successors;
use std::num::NonZeroUsize;
use time::{Date, Month, Weekday, Weekday::*};

const DAYS_IN_WEEK: usize = 7;

pub(super) trait WeekdayExt {
    fn index0(&self) -> u16;
}

impl WeekdayExt for Weekday {
    fn index0(&self) -> u16 {
        self.number_days_from_sunday().into()
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
// Invariant: At least one element of the array is Some
pub(super) struct Week([Option<StyledDate>; DAYS_IN_WEEK]);

impl Week {
    fn new(date: StyledDate) -> Self {
        let mut this = Week([None; DAYS_IN_WEEK]);
        this.set(date);
        this
    }

    fn set(&mut self, date: StyledDate) {
        let i = usize::from(date.date.weekday().index0());
        assert!(i < DAYS_IN_WEEK);
        self.0[i] = Some(date);
    }

    pub(super) fn enumerate(&self) -> EnumerateWeek<'_> {
        EnumerateWeek::new(self)
    }

    pub(super) fn get(&self, wd: Weekday) -> Option<StyledDate> {
        self.0.get(usize::from(wd.index0())).copied().flatten()
    }

    pub(super) fn has_month_start(&self) -> bool {
        self.0.iter().flatten().any(|sd| sd.date.day() == 1)
    }

    pub(super) fn first_ym(&self) -> (i32, Month) {
        self.0
            .iter()
            .flatten()
            .map(|sd| (sd.year(), sd.month()))
            .next()
            .expect("Week should contain at least one Some")
    }

    pub(super) fn last_ym(&self) -> (i32, Month) {
        self.0
            .iter()
            .flatten()
            .map(|sd| (sd.year(), sd.month()))
            .last()
            .expect("Week should contain at least one Some")
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
        loop {
            let Some(wd) = self.next_weekday else {
                return None;
            };
            self.next_weekday = match wd.next() {
                Sunday => None,
                wd2 => Some(wd2),
            };
            if let Some(date) = self.week.get(wd) {
                return Some((wd, date));
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(super) struct WeekFactory<S>(S);

impl<S: DateStyler> WeekFactory<S> {
    pub(super) fn new(styler: S) -> Self {
        WeekFactory(styler)
    }

    pub(super) fn around_date(&self, date: Date, week_qty: NonZeroUsize) -> VecDeque<Week> {
        let mut weeks = VecDeque::with_capacity(week_qty.get() + 1);
        let start_week = self.make(date);
        weeks.push_front(start_week);
        for w in self
            .iter_weeks_before(start_week)
            .take((week_qty.get() - 1) / 2)
        {
            weeks.push_front(w);
        }
        weeks.extend(
            self.iter_weeks_after(start_week)
                .take(week_qty.get() - weeks.len()),
        );
        if weeks.len() < week_qty.get() {
            // We are near the end of time, and so the "after" weeks were
            // short.  Fill towards the past.
            for w in self
                .iter_weeks_before(start_week)
                .take(week_qty.get() - weeks.len())
            {
                weeks.push_front(w);
            }
        }
        weeks
    }

    fn style_date(&self, date: Date) -> StyledDate {
        StyledDate {
            date,
            style: self.0.date_style(date),
        }
    }

    // Returns the Week containing the given date, which can be at any day of
    // the week
    fn make(&self, date: Date) -> Week {
        let i = usize::from(date.weekday().index0());
        let mut week = Week::new(self.style_date(date));
        for d in iter_days_before(date).take(i) {
            week.set(self.style_date(d));
        }
        for d in iter_days_after(date).take(DAYS_IN_WEEK - i - 1) {
            week.set(self.style_date(d));
        }
        week
    }

    pub(super) fn week_before(&self, week: &Week) -> Option<Week> {
        week.get(Sunday)
            .and_then(|sd| sd.date.previous_day())
            .map(|d| self.make(d))
    }

    pub(super) fn week_after(&self, week: &Week) -> Option<Week> {
        week.get(Saturday)
            .and_then(|sd| sd.date.next_day())
            .map(|d| self.make(d))
    }

    fn iter_weeks_before(&self, week: Week) -> impl Iterator<Item = Week> + '_ {
        successors(Some(week), |w| self.week_before(w)).skip(1)
    }

    fn iter_weeks_after(&self, week: Week) -> impl Iterator<Item = Week> + '_ {
        successors(Some(week), |w| self.week_after(w)).skip(1)
    }

    // Returns `None` if there are no weeks before `week`.  If there are weeks
    // before `week`, but not `qty` of them, only as many weeks as possible are
    // returned.
    pub(super) fn weeks_before(&self, week: Week, qty: NonZeroUsize) -> Option<VecDeque<Week>> {
        let mut iter = self.iter_weeks_before(week);
        let first_week = iter.next()?;
        let mut weeks = VecDeque::with_capacity(qty.get() + 1);
        weeks.push_front(first_week);
        for w in iter.take(qty.get() - 1) {
            weeks.push_front(w);
        }
        Some(weeks)
    }

    // Returns `None` if there are no weeks after `week`.  If there are weeks
    // after `week`, but not `qty` of them, only as many weeks as possible are
    // returned.
    pub(super) fn weeks_after(&self, week: Week, qty: NonZeroUsize) -> Option<VecDeque<Week>> {
        let mut iter = self.iter_weeks_after(week);
        let first_week = iter.next()?;
        let mut weeks = VecDeque::with_capacity(qty.get() + 1);
        weeks.push_back(first_week);
        weeks.extend(iter.take(qty.get() - 1));
        Some(weeks)
    }
}

fn iter_days_after(date: Date) -> impl Iterator<Item = Date> {
    successors(Some(date), |&d| d.next_day()).skip(1)
}

fn iter_days_before(date: Date) -> impl Iterator<Item = Date> {
    successors(Some(date), |&d| d.previous_day()).skip(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    struct NullStyler;

    impl DateStyler for NullStyler {
        fn date_style(&self, _date: Date) -> Style {
            Style::new()
        }
    }

    #[test]
    fn test_make() {
        let factory = WeekFactory::new(NullStyler);
        let week = factory.make(date!(2023 - 11 - 16));
        let mut iter = week.enumerate().map(|(wd, sd)| (wd, sd.date));
        assert_eq!(iter.next(), Some((Sunday, date!(2023 - 11 - 12))));
        assert_eq!(iter.next(), Some((Monday, date!(2023 - 11 - 13))));
        assert_eq!(iter.next(), Some((Tuesday, date!(2023 - 11 - 14))));
        assert_eq!(iter.next(), Some((Wednesday, date!(2023 - 11 - 15))));
        assert_eq!(iter.next(), Some((Thursday, date!(2023 - 11 - 16))));
        assert_eq!(iter.next(), Some((Friday, date!(2023 - 11 - 17))));
        assert_eq!(iter.next(), Some((Saturday, date!(2023 - 11 - 18))));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_make_from_sunday() {
        let factory = WeekFactory::new(NullStyler);
        let week = factory.make(date!(2023 - 11 - 12));
        let mut iter = week.enumerate().map(|(wd, sd)| (wd, sd.date));
        assert_eq!(iter.next(), Some((Sunday, date!(2023 - 11 - 12))));
        assert_eq!(iter.next(), Some((Monday, date!(2023 - 11 - 13))));
        assert_eq!(iter.next(), Some((Tuesday, date!(2023 - 11 - 14))));
        assert_eq!(iter.next(), Some((Wednesday, date!(2023 - 11 - 15))));
        assert_eq!(iter.next(), Some((Thursday, date!(2023 - 11 - 16))));
        assert_eq!(iter.next(), Some((Friday, date!(2023 - 11 - 17))));
        assert_eq!(iter.next(), Some((Saturday, date!(2023 - 11 - 18))));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_make_from_saturday() {
        let factory = WeekFactory::new(NullStyler);
        let week = factory.make(date!(2023 - 11 - 18));
        let mut iter = week.enumerate().map(|(wd, sd)| (wd, sd.date));
        assert_eq!(iter.next(), Some((Sunday, date!(2023 - 11 - 12))));
        assert_eq!(iter.next(), Some((Monday, date!(2023 - 11 - 13))));
        assert_eq!(iter.next(), Some((Tuesday, date!(2023 - 11 - 14))));
        assert_eq!(iter.next(), Some((Wednesday, date!(2023 - 11 - 15))));
        assert_eq!(iter.next(), Some((Thursday, date!(2023 - 11 - 16))));
        assert_eq!(iter.next(), Some((Friday, date!(2023 - 11 - 17))));
        assert_eq!(iter.next(), Some((Saturday, date!(2023 - 11 - 18))));
        assert_eq!(iter.next(), None);
    }
}
