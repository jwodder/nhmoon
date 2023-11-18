use super::util::*;
use super::DateStyler;
use std::cmp::Ordering;
use std::collections::VecDeque;
use thiserror::Error;
use time::Date;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WeekWindow<S> {
    pub(super) today: Date,
    start_date: Date,
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

    pub(super) fn ensure_weeks(&mut self, week_qty: usize) -> &VecDeque<Week> {
        // If we're asked to create zero weeks, create one week instead so that
        // `weeks` is always nonempty:
        let week_qty = week_qty.max(1);
        if let Some(weeks) = self.weeks.as_mut() {
            match weeks.len().cmp(&week_qty) {
                Ordering::Less => {
                    let mut extension = self
                        .week_factory
                        .weeks_after(
                            *weeks
                                .back()
                                .expect("WeekWindow.weeks should always be nonempty"),
                            week_qty - weeks.len(),
                        )
                        .unwrap_or_default();
                    weeks.append(&mut extension);
                    let missing = week_qty - weeks.len();
                    if missing > 0 {
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
        assert!(!weeks.is_empty());
        if let Some(w) = self.week_factory.week_after(weeks.back().unwrap()) {
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
        assert!(!weeks.is_empty());
        if let Some(w) = self.week_factory.week_before(weeks.front().unwrap()) {
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
        assert!(!weeks.is_empty());
        if let Some(mut page) = self
            .week_factory
            .weeks_after(*weeks.back().unwrap(), weeks.len())
        {
            if page.len() == weeks.len() {
                *weeks = page;
            } else {
                assert!(page.len() < weeks.len());
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
        assert!(!weeks.is_empty());
        if let Some(mut page) = self
            .week_factory
            .weeks_before(*weeks.front().unwrap(), weeks.len())
        {
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
