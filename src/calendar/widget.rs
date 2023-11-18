use super::util::WeekdayExt;
use super::weeks::WeekWindow;
use super::DateStyler;
use ratatui::{prelude::*, widgets::*};
use std::marker::PhantomData;
use time::{
    Month::{self, January},
    Weekday::{self, Saturday, Sunday},
};

static HEADER: &str = " Su     Mo     Tu     We     Th     Fr     Sa ";

/// Width of the calendar in columns, not counting the year and months in the
/// margins
const MAIN_WIDTH: u16 = 46;

/// Number of columns on the left side of the calendar, used as the margin in
/// which the year is written
const LEFT_MARGIN: u16 = 6;

const LONGEST_MONTH_NAME_LEN: u16 = 9; // September

/// Columns between the right edge of the calendar and the start of the month
/// name
const MONTH_GUTTER: u16 = 2;

/// Number of columns on the right side of the calendar, used as the margin in
/// which the month is written
const RIGHT_MARGIN: u16 = LONGEST_MONTH_NAME_LEN + MONTH_GUTTER;

const TOTAL_WIDTH: u16 = LEFT_MARGIN + MAIN_WIDTH + RIGHT_MARGIN;

/// Number of lines taken up by the header and its rule
const HEADER_LINES: u16 = 2;

/// Number of lines taken up by each week of the calendar
const WEEK_LINES: u16 = 2;

/// When inserting a vertical bar-like character between consecutive days in
/// the same week but different months, draw it this many columns to the right
/// of the left edge of the day on the left.
const VBAR_OFFSET: u16 = 5;

/// Number of columns per day of week
const DAY_WIDTH: u16 = 7;

const ACS_HLINE: char = '─';
const ACS_VLINE: char = '│';
const ACS_TTEE: char = '┬';
const ACS_ULCORNER: char = '┌';
const ACS_LRCORNER: char = '┘';

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Calendar<S> {
    _data: PhantomData<S>,
}

impl<S> Calendar<S> {
    pub(crate) fn new() -> Calendar<S> {
        Calendar { _data: PhantomData }
    }

    fn weeks_for_lines(lines: u16) -> usize {
        // ceil((lines - HEADER_LINES)/2)
        (lines.saturating_sub(HEADER_LINES).saturating_add(1) / 2).into()
    }
}

impl<S: DateStyler> StatefulWidget for Calendar<S> {
    type State = WeekWindow<S>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let left = (area.width.saturating_sub(MAIN_WIDTH) / 2).max(LEFT_MARGIN) - LEFT_MARGIN;
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(left),
                Constraint::Length(TOTAL_WIDTH.min(area.width)),
                Constraint::Min(0),
            ])
            .split(area);
        let area = chunks[1];
        let today = state.today;
        let weeks = state.ensure_weeks(Self::weeks_for_lines(area.height));
        let mut canvas = BufferCanvas::new(area, buf);
        canvas.draw_header();
        let top = weeks[0];
        canvas.draw_year(0, top[Sunday].year());
        canvas.draw_month(0, top[Saturday].month());
        for (i, week) in std::iter::zip(0u16.., weeks) {
            if week[Saturday].in_first_week_of_month() {
                canvas.draw_month(i, week[Saturday].month());
                if week[Saturday].month() == January {
                    if week[Sunday].month() == January {
                        canvas.draw_year(i, week[Sunday].year());
                    } else if usize::from(i + 1) < weeks.len() {
                        canvas.draw_year(i + 1, week[Saturday].year());
                    }
                }
            }
            for (wd, date) in week.enumerate() {
                let s = date.show(date.date == today);
                canvas.draw_day(i, wd, s);
                if date.is_last_day_of_month() {
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

    fn draw_year(&mut self, week_no: u16, year: i32) {
        self.mvprint(
            week_no * WEEK_LINES + HEADER_LINES,
            0,
            year.to_string(),
            Some(Style::new().bold()),
        );
    }

    fn draw_month(&mut self, week_no: u16, month: Month) {
        self.mvprint(
            week_no * WEEK_LINES + HEADER_LINES,
            LEFT_MARGIN + MAIN_WIDTH + MONTH_GUTTER,
            month.to_string(),
            Some(Style::new().bold()),
        );
    }

    fn draw_day(&mut self, week_no: u16, wd: Weekday, s: Span<'_>) {
        self.mvprint(
            week_no * WEEK_LINES + HEADER_LINES,
            LEFT_MARGIN + DAY_WIDTH * wd.index0(),
            s.content,
            Some(s.style),
        );
    }

    // `week_no` and `wd` specify the "coordinates" of the last day of the
    // month after which the border is drawn
    fn draw_month_border(&mut self, week_no: u16, wd: Weekday) {
        let y = week_no * WEEK_LINES + HEADER_LINES;
        let offset = DAY_WIDTH * wd.index0();
        let bar_col = LEFT_MARGIN + offset + VBAR_OFFSET;
        if wd != Saturday {
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

    fn mvaddch(&mut self, y: u16, x: u16, ch: char) {
        if y < self.area.height && x < self.area.width {
            self.buf
                .get_mut(x + self.area.x, y + self.area.y)
                .set_char(ch);
        }
    }

    fn mvprint<S: AsRef<str>>(&mut self, y: u16, x: u16, s: S, style: Option<Style>) {
        if y < self.area.height && x < self.area.width {
            let text = Text::styled(s.as_ref(), style.unwrap_or_default());
            let width = u16::try_from(text.width()).unwrap_or(u16::MAX);
            // Using a Paragraph lets us truncate text that extends beyond the
            // calendar's area, though we need to be sure that the Rect passed
            // to the Paragraph is entirely within the frame lest a panic
            // result.
            Paragraph::new(text).render(
                Rect {
                    x: x + self.area.x,
                    y: y + self.area.y,
                    width: (self.area.width - x).min(width),
                    height: 1,
                },
                self.buf,
            );
        }
    }

    fn hline(&mut self, y: u16, x: u16, ch: char, length: u16) {
        self.mvprint(y, x, String::from(ch).repeat(length.into()), None)
    }
}
