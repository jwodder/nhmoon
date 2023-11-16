mod weeks;
mod widget;
use self::weeks::*;
pub(crate) use self::widget::CalPagerWidget;
use chrono::naive::NaiveDate;
use ratatui::style::Style;
use std::cmp::Ordering;
use std::collections::VecDeque;

pub(crate) trait DateStyler {
    fn date_style(&self, date: NaiveDate) -> Style;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CalPager<S> {
    today: NaiveDate,
    weeks: Option<VecDeque<Week>>,
    week_factory: WeekFactory<S>,
}

impl<S: DateStyler> CalPager<S> {
    pub(crate) fn new(today: NaiveDate, date_styler: S) -> Self {
        let week_factory = WeekFactory::new(date_styler);
        CalPager {
            today,
            week_factory,
            weeks: None,
        }
    }

    fn ensure_weeks(&mut self, week_qty: usize) -> &VecDeque<Week> {
        if let Some(weeks) = self.weeks.as_mut() {
            match weeks.len().cmp(&week_qty) {
                Ordering::Less => {
                    // TODO: Avoid this unwrap!
                    let mut extension = self
                        .week_factory
                        .weeks_after(*weeks.back().unwrap(), week_qty - weeks.len());
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
