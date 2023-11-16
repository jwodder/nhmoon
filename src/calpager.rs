mod weeks;
use self::weeks::*;
use chrono::{
    naive::NaiveDate,
    Month::{self, January},
    Weekday::{self, Sat, Sun},
};
use ratatui::{prelude::*, widgets::StatefulWidget};
use std::collections::VecDeque;
use std::marker::PhantomData;

static HEADER: &str = " Su     Mo     Tu     We     Th     Fr     Sa ";

/// Width of the calendar in columns, not counting the year and months in the
/// margins
const MAIN_WIDTH: usize = 46;

const TOTAL_WIDTH: usize = LEFT_MARGIN + MAIN_WIDTH + RIGHT_MARGIN;

/// Number of columns on the left side of the calendar, used as the margin in
/// which the year is written
const LEFT_MARGIN: usize = 6;

const LONGEST_MONTH_NAME_LEN: usize = 9; // September

/// Number of columns on the right side of the calendar, used as the margin in
/// which the month is written
const RIGHT_MARGIN: usize = LONGEST_MONTH_NAME_LEN + MONTH_GUTTER;

/// Columns between the right edge of the calendar and the start of the month
/// name
const MONTH_GUTTER: usize = 2;

/// Number of lines taken up by the header and its rule
const HEADER_LINES: usize = 2;

/// Number of lines taken up by each week of the calendar
const WEEK_LINES: usize = 2;

/// When inserting a vertical bar-like character between consecutive days in
/// the same week but different months, draw it this many columns to the right
/// of the left edge of the day on the left.
const VBAR_OFFSET: usize = 5;

/// Number of columns per day of week
const DAY_WIDTH: usize = 7;

const ACS_HLINE: char = '─';
const ACS_VLINE: char = '│';
const ACS_TTEE: char = '┬';
const ACS_ULCORNER: char = '┌';
const ACS_LRCORNER: char = '┘';

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
        self.weeks
            .get_or_insert_with(|| self.week_factory.around_date(self.today, week_qty))
        // TODO: Resize self.weeks if it doesn't match `week_qty`
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) struct CalPagerWidget<S> {
    _data: PhantomData<S>,
}

impl<S> CalPagerWidget<S> {
    pub(crate) fn new() -> CalPagerWidget<S> {
        CalPagerWidget { _data: PhantomData }
    }

    fn weeks_for_lines(lines: u16) -> usize {
        // ceil((lines - HEADER_LINES)/2)
        let lines = usize::from(lines);
        lines.saturating_sub(HEADER_LINES).saturating_add(1) / 2
    }
}

impl<S: DateStyler> StatefulWidget for CalPagerWidget<S> {
    type State = CalPager<S>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let left = (area.width.saturating_sub(MAIN_WIDTH as u16) / 2).max(LEFT_MARGIN as u16)
            - (LEFT_MARGIN as u16);
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(left),
                Constraint::Length((TOTAL_WIDTH as u16).min(area.width)),
                Constraint::Min(0),
            ])
            .split(area);
        let area = chunks[1];
        let today = state.today;
        let weeks = state.ensure_weeks(Self::weeks_for_lines(area.height));
        let mut canvas = BufferCanvas::new(area, buf);
        canvas.draw_header();
        let top = weeks[0];
        canvas.draw_year(0, top[Sun].year());
        canvas.draw_month(0, top[Sat].month());
        for (i, week) in weeks.iter().enumerate() {
            if week[Sat].in_first_week_of_month() {
                canvas.draw_month(i, week[Sat].month());
                if week[Sat].month() == January {
                    if week[Sun].month() == January {
                        canvas.draw_year(i, week[Sun].year());
                    } else if i + 1 < weeks.len() {
                        canvas.draw_year(i + 1, week[Sat].year());
                    }
                }
            }
            for wd in WeekdayIter::new() {
                let s = week[wd].show(week[wd].date == today);
                canvas.draw_day(i, wd, s);
                if week[wd].is_last_day_of_month() {
                    canvas.draw_month_border(i, wd);
                }
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct BufferCanvas<'a> {
    area: Rect,
    buf: &'a mut Buffer,
}

impl<'a> BufferCanvas<'a> {
    fn new(area: Rect, buf: &'a mut Buffer) -> Self {
        Self { area, buf }
    }

    fn draw_header(&mut self) {
        self.mvprint(0, LEFT_MARGIN, HEADER, Some(Style::new().bold()));
        self.hline(1, LEFT_MARGIN, ACS_HLINE, MAIN_WIDTH);
    }

    fn draw_year(&mut self, week_no: usize, year: i32) {
        self.mvprint(
            week_no * WEEK_LINES + HEADER_LINES,
            0,
            year.to_string(),
            Some(Style::new().bold()),
        );
    }

    fn draw_month(&mut self, week_no: usize, month: Month) {
        self.mvprint(
            week_no * WEEK_LINES + HEADER_LINES,
            LEFT_MARGIN + MAIN_WIDTH + MONTH_GUTTER,
            month.name(),
            Some(Style::new().bold()),
        );
    }

    fn draw_day(&mut self, week_no: usize, wd: Weekday, s: Span<'_>) {
        self.mvprint(
            week_no * WEEK_LINES + HEADER_LINES,
            LEFT_MARGIN + DAY_WIDTH * wd.index0(),
            s.content,
            Some(s.style),
        );
    }

    // `week_no` and `wd` specify the "coordinates" of the last day of the
    // month after which the border is drawn
    fn draw_month_border(&mut self, week_no: usize, wd: Weekday) {
        let y = week_no * WEEK_LINES + HEADER_LINES;
        let offset = DAY_WIDTH * wd.index0();
        let bar_col = LEFT_MARGIN + offset + VBAR_OFFSET;
        if wd != Sat {
            self.mvaddch(y, bar_col, ACS_VLINE);
            self.mvaddch(
                y - 1,
                bar_col,
                if week_no == 0 { ACS_TTEE } else { ACS_ULCORNER },
            );
            if week_no > 0 {
                if let Some(length) = MAIN_WIDTH.checked_sub(offset + VBAR_OFFSET + 1) {
                    self.hline(y - 1, bar_col + 1, ACS_HLINE, length);
                }
            }
            self.mvaddch(y + 1, bar_col, ACS_LRCORNER);
        }
        self.hline(y + 1, LEFT_MARGIN, ACS_HLINE, offset + VBAR_OFFSET);
    }

    fn mvaddch(&mut self, y: usize, x: usize, ch: char) {
        let Ok(y) = u16::try_from(y) else {
            return;
        };
        let Ok(x) = u16::try_from(x) else {
            return;
        };
        if y < self.area.height && x < self.area.width {
            self.buf
                .get_mut(x + self.area.x, y + self.area.y)
                .set_char(ch);
        }
    }

    fn mvprint<S: AsRef<str>>(&mut self, y: usize, x: usize, s: S, style: Option<Style>) {
        let Ok(y) = u16::try_from(y) else {
            return;
        };
        let Ok(x) = u16::try_from(x) else {
            return;
        };
        // TODO: Guard against any part of the string being out of the buffer's
        // area, which causes a panic
        if y < self.area.height && x < self.area.width {
            self.buf.set_string(
                x + self.area.x,
                y + self.area.y,
                s,
                style.unwrap_or_default(),
            );
        }
    }

    fn hline(&mut self, y: usize, x: usize, ch: char, length: usize) {
        self.mvprint(y, x, String::from(ch).repeat(length), None)
    }
}
