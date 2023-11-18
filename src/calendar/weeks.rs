use super::util::*;
use super::DateStyler;
use std::cmp::Ordering;
use std::collections::VecDeque;
use time::Date;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WeekWindow<S> {
    pub(super) today: Date,
    weeks: Option<VecDeque<Week>>,
    week_factory: WeekFactory<S>,
}

impl<S: DateStyler> WeekWindow<S> {
    pub(crate) fn new(today: Date, date_styler: S) -> Self {
        let week_factory = WeekFactory::new(date_styler);
        WeekWindow {
            today,
            week_factory,
            weeks: None,
        }
    }

    pub(super) fn ensure_weeks(&mut self, week_qty: usize) -> &VecDeque<Week> {
        // If we're asked to create zero weeks, create one week instead so that
        // `weeks` is always nonempty:
        let week_qty = week_qty.max(1);
        if let Some(weeks) = self.weeks.as_mut() {
            match weeks.len().cmp(&week_qty) {
                Ordering::Less => {
                    let mut extension = self.week_factory.weeks_after(
                        *weeks.back().expect("weeks should always be nonempty"),
                        week_qty - weeks.len(),
                    );
                    weeks.append(&mut extension);
                }
                Ordering::Greater => weeks.truncate(week_qty),
                Ordering::Equal => (),
            }
        }
        self.weeks
            .get_or_insert_with(|| self.week_factory.around_date(self.today, week_qty))
    }

    pub(crate) fn jump_to_today(&mut self) {
        if let Some(weeks) = self.weeks.as_mut() {
            *weeks = self.week_factory.around_date(self.today, weeks.len());
        }
    }

    pub(crate) fn one_week_forwards(&mut self) {
        if let Some(weeks) = self.weeks.as_mut() {
            if let Some(w) = weeks.back().map(|w| self.week_factory.week_after(w)) {
                weeks.push_back(w);
                weeks.pop_front();
            }
        }
    }

    pub(crate) fn one_week_backwards(&mut self) {
        if let Some(weeks) = self.weeks.as_mut() {
            if let Some(w) = weeks.front().map(|w| self.week_factory.week_before(w)) {
                weeks.push_front(w);
                weeks.pop_back();
            }
        }
    }

    pub(crate) fn one_page_forwards(&mut self) {
        if let Some(weeks) = self.weeks.as_mut() {
            if let Some(w) = weeks.back().copied() {
                *weeks = self.week_factory.weeks_after(w, weeks.len());
            }
        }
    }

    pub(crate) fn one_page_backwards(&mut self) {
        if let Some(weeks) = self.weeks.as_mut() {
            if let Some(w) = weeks.front().copied() {
                *weeks = self.week_factory.weeks_before(w, weeks.len());
            }
        }
    }
}