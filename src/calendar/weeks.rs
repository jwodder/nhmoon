use super::util::*;
use super::DateStyler;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::num::NonZeroUsize;
use thiserror::Error;
use time::Date;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WeekWindow<S> {
    pub(super) today: Date,
    start_date: Date,
    // Invariant: When `weeks` is `Some`, it is always nonempty, and thus going
    // one page forwards/backwards always does something.
    weeks: Option<VecDeque<Week>>,
    week_factory: WeekFactory<S>,
}

impl<S: DateStyler> WeekWindow<S> {
    pub(crate) fn new(today: Date, date_styler: S) -> Self {
        let week_factory = WeekFactory::new(date_styler);
        WeekWindow {
            today,
            start_date: today,
            week_factory,
            weeks: None,
        }
    }

    pub(crate) fn start_date(mut self, date: Date) -> Self {
        self.start_date = date;
        self
    }

    pub(super) fn ensure_weeks(&mut self, week_qty: NonZeroUsize) -> &VecDeque<Week> {
        if let Some(weeks) = self.weeks.as_mut() {
            match weeks.len().cmp(&week_qty.get()) {
                Ordering::Less => {
                    let missing = NonZeroUsize::new(week_qty.get() - weeks.len())
                        .expect("Greater minus lesser should be nonzero");
                    let mut extension = self
                        .week_factory
                        .weeks_after(
                            *weeks
                                .back()
                                .expect("WeekWindow.weeks should always be nonempty"),
                            missing,
                        )
                        .unwrap_or_default();
                    weeks.append(&mut extension);
                    if let Some(missing) = NonZeroUsize::new(week_qty.get() - weeks.len()) {
                        // The terminal was heightened while at the end of
                        // time, so "scroll" the calendar down by prepending
                        // weeks from before the window.
                        if let Some(prextension) = self.week_factory.weeks_before(
                            *weeks
                                .front()
                                .expect("WeekWindow.weeks should always be nonempty"),
                            missing,
                        ) {
                            for w in prextension {
                                weeks.push_front(w);
                            }
                        }
                    }
                }
                Ordering::Greater => weeks.truncate(week_qty.get()),
                Ordering::Equal => (),
            }
        }
        self.weeks
            .get_or_insert_with(|| self.week_factory.around_date(self.start_date, week_qty))
    }

    pub(crate) fn jump_to_today(&mut self) {
        if let Some(weeks) = self.weeks.as_mut() {
            let week_qty = NonZeroUsize::new(weeks.len()).expect("weeks.len() should be nonzero");
            *weeks = self.week_factory.around_date(self.today, week_qty);
        }
    }

    pub(crate) fn one_week_forwards(&mut self) -> Result<(), OutOfTimeError> {
        let Some(weeks) = self.weeks.as_mut() else {
            return Ok(());
        };
        let back = weeks
            .back()
            .expect("WeekWindow.weeks should always be nonempty");
        if let Some(w) = self.week_factory.week_after(back) {
            weeks.push_back(w);
            weeks.pop_front();
            Ok(())
        } else {
            Err(OutOfTimeError)
        }
    }

    pub(crate) fn one_week_backwards(&mut self) -> Result<(), OutOfTimeError> {
        let Some(weeks) = self.weeks.as_mut() else {
            return Ok(());
        };
        let front = weeks
            .front()
            .expect("WeekWindow.weeks should always be nonempty");
        if let Some(w) = self.week_factory.week_before(front) {
            weeks.push_front(w);
            weeks.pop_back();
            Ok(())
        } else {
            Err(OutOfTimeError)
        }
    }

    pub(crate) fn one_page_forwards(&mut self) -> Result<(), OutOfTimeError> {
        let Some(weeks) = self.weeks.as_mut() else {
            return Ok(());
        };
        let week_qty = NonZeroUsize::new(weeks.len()).expect("weeks.len() should be nonzero");
        let back = weeks
            .back()
            .expect("WeekWindow.weeks should always be nonempty");
        if let Some(mut page) = self.week_factory.weeks_after(*back, week_qty) {
            if page.len() == weeks.len() {
                *weeks = page;
            } else {
                assert!(
                    page.len() < weeks.len(),
                    "week_after() should not return more than week_qty items"
                );
                weeks.drain(0..(page.len()));
                weeks.append(&mut page);
            }
            Ok(())
        } else {
            Err(OutOfTimeError)
        }
    }

    pub(crate) fn one_page_backwards(&mut self) -> Result<(), OutOfTimeError> {
        let Some(weeks) = self.weeks.as_mut() else {
            return Ok(());
        };
        let week_qty = NonZeroUsize::new(weeks.len()).expect("weeks.len() should be nonzero");
        let front = weeks
            .front()
            .expect("WeekWindow.weeks should always be nonempty");
        if let Some(mut page) = self.week_factory.weeks_before(*front, week_qty) {
            if page.len() < weeks.len() {
                weeks.truncate(weeks.len() - page.len());
                page.append(weeks);
            }
            *weeks = page;
            Ok(())
        } else {
            Err(OutOfTimeError)
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, Error, PartialEq)]
#[error("reached the end of time")]
pub(crate) struct OutOfTimeError;
