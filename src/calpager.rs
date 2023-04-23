use chrono::{naive::NaiveDate, Datelike, Local, Month, Weekday, Weekday::*};
use crossterm::cursor::MoveTo;
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::{
    style, Color, ContentStyle, Print, PrintStyledContent, StyledContent, Stylize,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use crossterm::{queue, ExecutableCommand, QueueableCommand};
use num_traits::cast::FromPrimitive;
use std::collections::VecDeque;
use std::fmt::Display;
use std::io::Write;
use std::ops::{Deref, Index, Range};

static HEADER: &str = " Su     Mo     Tu     We     Th     Fr     Sa";

const DAY_WIDTH: u16 = 7;

const ACS_HLINE: char = '\u{2500}';
const ACS_VLINE: char = '\u{2502}';
const ACS_TTEE: char = '\u{252C}';
const ACS_ULCORNER: char = '\u{250C}';
const ACS_LRCORNER: char = '\u{2518}';

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct StyledDay {
    date: NaiveDate,
    style: ContentStyle,
}

impl StyledDay {
    fn month_name(&self) -> &'static str {
        Month::from_u32(self.date.month()).unwrap().name()
    }

    fn apply_style<D: Display>(&self, val: D) -> StyledContent<D> {
        self.style.apply(val)
    }
}

impl Deref for StyledDay {
    type Target = NaiveDate;

    fn deref(&self) -> &NaiveDate {
        &self.date
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Week([StyledDay; 7]);

impl Week {
    fn new<F: FnMut(NaiveDate) -> ContentStyle>(sunday: NaiveDate, mut highlighter: F) -> Self {
        Week(std::array::from_fn(move |i| {
            let date = n_days_after(sunday, i);
            let style = highlighter(date);
            StyledDay { date, style }
        }))
    }
}

impl Index<Weekday> for Week {
    type Output = StyledDay;

    fn index(&self, index: Weekday) -> &StyledDay {
        &self.0[weekday_index(index)]
    }
}

pub struct Screen<W: Write> {
    writer: W,
    altscreen: bool,
    raw: bool,
    fgcolor: Option<Color>,
    bgcolor: Option<Color>,
}

impl<W: Write> Screen<W> {
    pub fn new(writer: W) -> Screen<W> {
        Screen {
            writer,
            altscreen: false,
            raw: false,
            fgcolor: None,
            bgcolor: None,
        }
    }

    pub fn altscreen(&mut self) -> crossterm::Result<&mut Self> {
        self.writer.execute(EnterAlternateScreen)?;
        self.altscreen = true;
        Ok(self)
    }

    pub fn raw(&mut self) -> crossterm::Result<&mut Self> {
        enable_raw_mode()?;
        self.raw = true;
        Ok(self)
    }

    pub fn set_fg_color(&mut self, color: Color) -> &mut Self {
        self.fgcolor = Some(color);
        self
    }

    // fill_clear() must be called after this in order to actually apply the
    // background color to the entire screen
    pub fn set_bg_color(&mut self, color: Color) -> &mut Self {
        self.bgcolor = Some(color);
        self
    }

    fn apply_fgbg<D: Display>(&self, mut content: StyledContent<D>) -> StyledContent<D> {
        let style = content.style_mut();
        if style.foreground_color.is_none() && self.fgcolor.is_some() {
            style.foreground_color = self.fgcolor;
        }
        if style.background_color.is_none() && self.bgcolor.is_some() {
            style.background_color = self.bgcolor;
        }
        content
    }

    pub fn mvprint<S, D>(&mut self, y: u16, x: u16, s: S) -> crossterm::Result<()>
    where
        S: Stylize<Styled = StyledContent<D>>,
        D: Display,
    {
        let s = self.apply_fgbg(s.stylize());
        queue!(self.writer, MoveTo(x, y), PrintStyledContent(s))
    }

    pub fn addch(&mut self, ch: char) -> crossterm::Result<()> {
        self.writer.queue(Print(self.apply_fgbg(style(ch))))?;
        Ok(())
    }

    pub fn hline(&mut self, y: u16, x: u16, ch: char, length: usize) -> crossterm::Result<()> {
        self.mvprint(y, x, String::from(ch).repeat(length))
    }

    pub fn beep(&mut self) -> crossterm::Result<()> {
        self.writer.execute(Print("\x07"))?;
        Ok(())
    }

    pub fn moveto(&mut self, y: u16, x: u16) -> crossterm::Result<()> {
        self.writer.queue(MoveTo(x, y))?;
        Ok(())
    }

    pub fn refresh(&mut self) -> crossterm::Result<()> {
        self.writer.flush()
    }

    pub fn fill_clear(&mut self) -> crossterm::Result<()> {
        self.writer.queue(Clear(ClearType::All))?;
        let (cols, lines) = size()?;
        let s = " ".repeat(cols.into());
        let blankline = if let Some(bg) = self.bgcolor {
            s.as_str().on(bg)
        } else {
            s.as_str().stylize()
        };
        for y in 0..lines {
            self.mvprint(y, 0, blankline)?;
        }
        Ok(())
    }
}

impl<W: Write> Drop for Screen<W> {
    fn drop(&mut self) {
        if self.raw {
            let _ = disable_raw_mode();
        }
        if self.altscreen {
            let _ = self.writer.execute(LeaveAlternateScreen);
        }
    }
}

struct CalPager<W: Write, F> {
    screen: Screen<W>,
    today: NaiveDate,
    weeks: WeekSheet<F>,
    left: u16,
    rows: u16,
    cols: u16,
    lines: u16,
}

impl<W: Write, F: FnMut(NaiveDate) -> ContentStyle> CalPager<W, F> {
    fn new(screen: Screen<W>, highlighter: F) -> crossterm::Result<Self> {
        let (cols, lines) = size()?;
        let rows = (lines - 1) / 2; // ceil((lines - 2)/2)
        let left = (cols - 46) / 2;
        let today = Local::now().date_naive();
        let weeks = WeekSheet::new(today, rows.into(), highlighter);
        Ok(CalPager {
            today,
            screen,
            weeks,
            left,
            rows,
            cols,
            lines,
        })
    }

    fn run(&mut self) -> crossterm::Result<()> {
        let normal_key_mods = KeyModifiers::NONE | KeyModifiers::SHIFT;
        loop {
            self.draw()?;
            match read()? {
                Event::Key(KeyEvent {
                    code, modifiers, ..
                }) if normal_key_mods.contains(modifiers) => match code {
                    KeyCode::Char('j') | KeyCode::PageDown => self.scroll_down(),
                    KeyCode::Char('k') | KeyCode::PageUp => self.scroll_up(),
                    KeyCode::Char('0') => self.reset(),
                    KeyCode::Char('z') => {
                        for _ in 0..self.rows {
                            self.scroll_down();
                        }
                    }
                    KeyCode::Char('w') => {
                        for _ in 0..self.rows {
                            self.scroll_up();
                        }
                    }
                    KeyCode::Char('q') => break,
                    _ => self.screen.beep()?,
                },
                _ => self.screen.beep()?,
            }
        }
        Ok(())
    }

    fn draw(&mut self) -> crossterm::Result<()> {
        self.screen.fill_clear()?;
        self.screen.mvprint(0, self.left, HEADER.bold())?;
        self.screen.hline(1, self.left, ACS_HLINE, 46)?;
        let top = self.weeks.top();
        self.screen
            .mvprint(2, self.left - 6, style(top[Sun].year()).bold())?;
        self.screen
            .mvprint(2, self.left + 48, top[Sat].month_name().bold())?;
        for (i, week) in (0..).zip(self.weeks.visible_weeks()) {
            let y = 2 + i * 2;
            if week[Sat].month() != week.pred()[Sat].month() {
                self.screen
                    .mvprint(y, self.left + 48, week[Sat].month_name().bold())?;
                if week[Sat].month() == 1 {
                    if week[Sun].month() == 1 {
                        self.screen
                            .mvprint(y, self.left - 6, style(week[Sun].year()).bold())?;
                    } else if i < self.rows - 1 {
                        self.screen.mvprint(
                            y + 2,
                            self.left - 6,
                            style(week[Sat].year()).bold(),
                        )?;
                    }
                }
            }
            for (j, wd) in (0..).zip(WeekdayIter::new()) {
                let s = if week[wd].date == self.today {
                    format!(" [{:2}] ", week[wd].day())
                } else {
                    format!("  {:2}  ", week[wd].day())
                };
                self.screen
                    .mvprint(y, self.left - 1 + DAY_WIDTH * j, week[wd].apply_style(s))?;
                let mut end_of_border = false;
                if j < 6 && week[wd].month() != week[wd.succ()].month() {
                    self.screen.addch(ACS_VLINE)?;
                    self.screen.mvprint(
                        y - 1,
                        self.left + 5 + DAY_WIDTH * j,
                        if i == 0 { ACS_TTEE } else { ACS_ULCORNER },
                    )?;
                    if i < self.rows - 1 {
                        self.screen
                            .mvprint(y + 1, self.left + 5 + DAY_WIDTH * j, ACS_LRCORNER)?;
                    }
                    end_of_border = true;
                } else {
                    self.screen.addch(' ')?;
                }
                if i < self.rows - 1 && week[wd].month() != week.succ()[wd].month() {
                    self.screen.hline(
                        y + 1,
                        self.left - 1 + DAY_WIDTH * j + u16::from(j == 0),
                        ACS_HLINE,
                        if wd == Sat {
                            5
                        } else {
                            7 - usize::from(end_of_border) - usize::from(j == 0)
                        },
                    )?;
                }
            }
        }
        self.screen.moveto(self.lines - 1, self.cols - 1)?;
        self.screen.refresh()
    }

    fn scroll_up(&mut self) {
        self.weeks.scroll_up();
    }

    fn scroll_down(&mut self) {
        self.weeks.scroll_down();
    }

    fn reset(&mut self) {
        self.weeks.jump_to(self.today);
    }
}

struct WeekSheet<F> {
    week_factory: WeekFactory<F>,
    rows: usize,
    capacity: usize,
    data: VecDeque<Week>,
    top_index: usize,
}

// IMPORTANT: In order for WeekCursor::{succ,pred}() to work, there must always
// be at least one Week available before `top_index` and at least one week at
// or after `top_index + rows`.
impl<F: FnMut(NaiveDate) -> ContentStyle> WeekSheet<F> {
    fn new(start_date: NaiveDate, rows: usize, highlighter: F) -> Self {
        let mut week_factory = WeekFactory::new(highlighter);
        let capacity = rows * 3;
        let mut data = VecDeque::with_capacity(capacity);
        Self::populate(&mut data, &mut week_factory, start_date, rows);
        WeekSheet {
            week_factory,
            rows,
            capacity,
            data,
            top_index: 1,
        }
    }

    fn populate(
        data: &mut VecDeque<Week>,
        week_factory: &mut WeekFactory<F>,
        date: NaiveDate,
        rows: usize,
    ) {
        let sunday = n_days_before(date, weekday_index(date.weekday()));
        let start_week = week_factory.make(sunday);
        data.push_front(start_week);
        let mut week = start_week;
        for _ in 0..((rows / 2) + 1) {
            week = week_factory.week_before(&week);
            data.push_front(week);
        }
        week = start_week;
        for _ in 0..((rows + 1) / 2 + 1) {
            week = week_factory.week_after(&week);
            data.push_back(week);
        }
    }

    fn scroll_up(&mut self) {
        if self.top_index == 1 {
            let new_top = self.week_factory.week_before(&self.data[0]);
            if self.data.len() >= self.capacity {
                self.data.pop_back();
            }
            self.data.push_front(new_top);
        } else {
            self.top_index -= 1;
        }
    }

    fn scroll_down(&mut self) {
        self.top_index += 1;
        if let Some(needed) = (self.top_index + self.rows + 1).checked_sub(self.data.len()) {
            let mut week = self.data.back().copied().unwrap();
            for _ in 0..needed {
                week = self.week_factory.week_after(&week);
                if self.data.len() >= self.capacity {
                    self.data.pop_front();
                    self.top_index -= 1;
                }
                self.data.push_back(week);
            }
        }
    }

    fn jump_to(&mut self, date: NaiveDate) {
        self.data.clear();
        Self::populate(&mut self.data, &mut self.week_factory, date, self.rows);
        self.top_index = 1;
    }

    fn top(&self) -> &Week {
        &self.data[self.top_index]
    }

    fn visible_weeks(&self) -> VisibleWeeks<'_, F> {
        VisibleWeeks::new(self)
    }
}

struct WeekFactory<F>(F);

impl<F: FnMut(NaiveDate) -> ContentStyle> WeekFactory<F> {
    fn new(highlighter: F) -> Self {
        WeekFactory(highlighter)
    }

    fn make(&mut self, sunday: NaiveDate) -> Week {
        Week::new(sunday, &mut self.0)
    }

    fn week_before(&mut self, week: &Week) -> Week {
        self.make(n_days_before(*week[Sun], 7))
    }

    fn week_after(&mut self, week: &Week) -> Week {
        self.make(n_days_after(*week[Sun], 7))
    }
}

struct VisibleWeeks<'a, F> {
    week_sheet: &'a WeekSheet<F>,
    inner: Range<usize>,
}

impl<'a, F> VisibleWeeks<'a, F> {
    fn new(week_sheet: &'a WeekSheet<F>) -> Self {
        let start = week_sheet.top_index;
        let end = start + week_sheet.rows;
        let inner = start..end;
        VisibleWeeks { week_sheet, inner }
    }
}

impl<'a, F> Iterator for VisibleWeeks<'a, F> {
    type Item = WeekCursor<'a, F>;

    fn next(&mut self) -> Option<WeekCursor<'a, F>> {
        let i = self.inner.next()?;
        Some(WeekCursor::new(self.week_sheet, i))
    }
}

struct WeekCursor<'a, F> {
    week_sheet: &'a WeekSheet<F>,
    index: usize,
}

impl<'a, F> WeekCursor<'a, F> {
    fn new(week_sheet: &'a WeekSheet<F>, index: usize) -> Self {
        WeekCursor { week_sheet, index }
    }

    fn succ(&self) -> &Week {
        &self.week_sheet.data[self.index + 1]
    }

    fn pred(&self) -> &Week {
        &self.week_sheet.data[self.index - 1]
    }
}

impl<'a, F> Deref for WeekCursor<'a, F> {
    type Target = Week;

    fn deref(&self) -> &Week {
        &self.week_sheet.data[self.index]
    }
}

struct WeekdayIter(Option<Weekday>);

impl WeekdayIter {
    fn new() -> Self {
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

pub fn calendar_pager<W: Write, F: FnMut(NaiveDate) -> ContentStyle>(
    screen: Screen<W>,
    highlighter: F,
) -> crossterm::Result<()> {
    CalPager::new(screen, highlighter)?.run()
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

fn weekday_index(wd: Weekday) -> usize {
    wd.num_days_from_sunday().try_into().unwrap()
}
