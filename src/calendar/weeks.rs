use super::util::{NonEmptyVecDeque, Week, WeekFactory};
use super::DateStyler;
use std::cmp::Ordering;
use std::num::NonZeroUsize;
use thiserror::Error;
use time::Date;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WeekWindow<S> {
    pub(super) today: Date,
    start_date: Date,
    weeks: Option<NonEmptyVecDeque<Week>>,
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

    pub(super) fn ensure_weeks(&mut self, week_qty: NonZeroUsize) -> &NonEmptyVecDeque<Week> {
        if let Some(weeks) = self.weeks.as_mut() {
            match weeks.len().cmp(&week_qty) {
                Ordering::Less => {
                    if let Some(mut extension) = nonzero_sub(week_qty, weeks.len())
                        .and_then(|missing| self.week_factory.weeks_after(*weeks.back(), missing))
                    {
                        weeks.append(&mut extension);
                    }
                    if let Some(missing) = nonzero_sub(week_qty, weeks.len()) {
                        // The terminal was heightened while at the end of
                        // time, so "scroll" the calendar down by prepending
                        // weeks from before the window.
                        if let Some(prextension) =
                            self.week_factory.weeks_before(*weeks.front(), missing)
                        {
                            weeks.prepend(prextension);
                        }
                    }
                }
                Ordering::Greater => weeks.truncate(week_qty),
                Ordering::Equal => (),
            }
        }
        self.weeks
            .get_or_insert_with(|| self.week_factory.around_date(self.start_date, week_qty))
    }

    pub(crate) fn jump_to_today(&mut self) {
        if let Some(weeks) = self.weeks.as_mut() {
            *weeks = self.week_factory.around_date(self.today, weeks.len());
        }
    }

    pub(crate) fn one_week_forwards(&mut self) -> Result<(), OutOfTimeError> {
        let Some(weeks) = self.weeks.as_mut() else {
            return Ok(());
        };
        if let Some(w) = self.week_factory.week_after(weeks.back()) {
            weeks.rotate_push_back(w);
            Ok(())
        } else {
            Err(OutOfTimeError)
        }
    }

    pub(crate) fn one_week_backwards(&mut self) -> Result<(), OutOfTimeError> {
        let Some(weeks) = self.weeks.as_mut() else {
            return Ok(());
        };
        if let Some(w) = self.week_factory.week_before(weeks.front()) {
            weeks.rotate_push_front(w);
            Ok(())
        } else {
            Err(OutOfTimeError)
        }
    }

    pub(crate) fn one_page_forwards(&mut self) -> Result<(), OutOfTimeError> {
        let Some(weeks) = self.weeks.as_mut() else {
            return Ok(());
        };
        let week_qty = weeks.len();
        if let Some(mut page) = self.week_factory.weeks_after(*weeks.back(), week_qty) {
            if page.len() == week_qty {
                *weeks = page;
            } else {
                assert!(
                    page.len() < week_qty,
                    "week_after() should not return more than week_qty items"
                );
                weeks.rotate_append(&mut page);
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
        let week_qty = weeks.len();
        if let Some(mut page) = self.week_factory.weeks_before(*weeks.front(), week_qty) {
            if let Some(len) = nonzero_sub(week_qty, page.len()) {
                weeks.truncate(len);
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

fn nonzero_sub(lhs: NonZeroUsize, rhs: NonZeroUsize) -> Option<NonZeroUsize> {
    NonZeroUsize::new(lhs.get() - rhs.get())
}
