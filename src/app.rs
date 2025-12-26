use crate::calendar::{Calendar, DateStyler, WeekWindow};
use crate::help::Help;
use crate::jumpto::{JumpTo, JumpToInput, JumpToOutput, JumpToState};
use crate::theme::BASE_STYLE;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, read};
use ratatui::{
    Terminal,
    backend::Backend,
    buffer::Buffer,
    layout::Rect,
    widgets::{StatefulWidget, Widget},
};
use std::io::{self, Write};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct App<S> {
    weeks: WeekWindow<S>,
    state: AppState,
}

impl<S: DateStyler> App<S> {
    pub(crate) fn new(weeks: WeekWindow<S>) -> App<S> {
        App {
            weeks,
            state: AppState::Calendar,
        }
    }

    pub(crate) fn run<B: Backend>(mut self, mut terminal: Terminal<B>) -> io::Result<()>
    where
        io::Error: From<B::Error>,
    {
        while !self.quitting() {
            self.draw(&mut terminal)?;
            self.handle_input()?;
        }
        Ok(())
    }

    fn draw<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()>
    where
        io::Error: From<B::Error>,
    {
        terminal.draw(|frame| frame.render_widget(self, frame.area()))?;
        Ok(())
    }

    fn handle_input(&mut self) -> io::Result<()> {
        let normal_modifiers = KeyModifiers::NONE | KeyModifiers::SHIFT;
        if let Some(KeyEvent {
            code, modifiers, ..
        }) = read()?.as_key_press_event()
        {
            if modifiers == KeyModifiers::CONTROL && code == KeyCode::Char('c') {
                self.state = AppState::Quitting;
            } else if !normal_modifiers.contains(modifiers) || !self.handle_key(code) {
                self.beep()?;
            }
        }
        // else: Redraw on resize, and we might as well redraw on other stuff
        // too
        Ok(())
    }

    // Returns `false` if the user pressed an invalid key
    fn handle_key(&mut self, key: KeyCode) -> bool {
        match &mut self.state {
            AppState::Calendar => match key {
                KeyCode::Char('j') | KeyCode::Down => self.scroll_down(),
                KeyCode::Char('k') | KeyCode::Up => self.scroll_up(),
                KeyCode::Char('z') | KeyCode::PageDown => self.page_down(),
                KeyCode::Char('w') | KeyCode::PageUp => self.page_up(),
                KeyCode::Char('0') | KeyCode::Home => {
                    self.reset();
                    true
                }
                KeyCode::Char('g') => {
                    self.state = AppState::Jumping(JumpToState::new());
                    true
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    self.state = AppState::Quitting;
                    true
                }
                KeyCode::Char('?') => {
                    self.state = AppState::Helping;
                    true
                }
                _ => false,
            },
            AppState::Helping => {
                self.state = AppState::Calendar;
                true
            }
            AppState::Jumping(state) => {
                if matches!(key, KeyCode::Char('q' | 'g') | KeyCode::Esc) {
                    self.state = AppState::Calendar;
                    true
                } else {
                    let output = match key {
                        KeyCode::Char('-') => state.handle_input(JumpToInput::Negative),
                        KeyCode::Char('+') => state.handle_input(JumpToInput::Positive),
                        KeyCode::Char('0') => state.handle_input(JumpToInput::Digit(0)),
                        KeyCode::Char('1') => state.handle_input(JumpToInput::Digit(1)),
                        KeyCode::Char('2') => state.handle_input(JumpToInput::Digit(2)),
                        KeyCode::Char('3') => state.handle_input(JumpToInput::Digit(3)),
                        KeyCode::Char('4') => state.handle_input(JumpToInput::Digit(4)),
                        KeyCode::Char('5') => state.handle_input(JumpToInput::Digit(5)),
                        KeyCode::Char('6') => state.handle_input(JumpToInput::Digit(6)),
                        KeyCode::Char('7') => state.handle_input(JumpToInput::Digit(7)),
                        KeyCode::Char('8') => state.handle_input(JumpToInput::Digit(8)),
                        KeyCode::Char('9') => state.handle_input(JumpToInput::Digit(9)),
                        KeyCode::Backspace | KeyCode::Delete => {
                            state.handle_input(JumpToInput::Backspace)
                        }
                        KeyCode::Enter => state.handle_input(JumpToInput::Enter),
                        _ => JumpToOutput::Invalid,
                    };
                    match output {
                        JumpToOutput::Ok => true,
                        JumpToOutput::Invalid => false,
                        JumpToOutput::Jump(date) => {
                            self.state = AppState::Calendar;
                            self.jump_to(date);
                            true
                        }
                    }
                }
            }
            AppState::Quitting => false,
        }
    }

    fn beep(&self) -> io::Result<()> {
        io::stdout().write_all(b"\x07")
    }

    fn quitting(&self) -> bool {
        self.state == AppState::Quitting
    }

    fn scroll_down(&mut self) -> bool {
        self.weeks.one_week_forwards().is_ok()
    }

    fn scroll_up(&mut self) -> bool {
        self.weeks.one_week_backwards().is_ok()
    }

    fn page_down(&mut self) -> bool {
        self.weeks.one_page_forwards().is_ok()
    }

    fn page_up(&mut self) -> bool {
        self.weeks.one_page_backwards().is_ok()
    }

    fn reset(&mut self) {
        self.weeks.jump_to_today();
    }

    fn jump_to(&mut self, date: time::Date) {
        self.weeks.jump_to_date(date);
    }
}

impl<S: DateStyler> Widget for &mut App<S> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, BASE_STYLE);
        let cal = Calendar::<S>::new();
        cal.render(area, buf, &mut self.weeks);
        if self.state == AppState::Helping {
            Help.render(area, buf);
        } else if let AppState::Jumping(ref mut state) = self.state {
            JumpTo.render(area, buf, state);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AppState {
    Calendar,
    Helping,
    Jumping(JumpToState),
    Quitting,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moon::Phoon;
    use crate::theme::{
        BASE_STYLE, FULL_MOON_STYLE, MONTH_STYLE, NEW_MOON_STYLE, WEEKDAY_STYLE, YEAR_STYLE,
    };

    #[test]
    fn test_across_year() {
        let today = time::Date::from_calendar_date(2025, time::Month::January, 22).unwrap();
        let calpager = WeekWindow::new(today, Phoon);
        let mut app = App::new(calpager);
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);
        app.render(area, &mut buffer);
        let mut expected = Buffer::with_lines([
            "                  Su     Mo     Tu     We     Th     Fr     Sa                  ",
            "                 ──────────────────────────────────────────────                 ",
            "           2024   15     16     17     18     19     20     21   December       ",
            "                                                                                ",
            "                  22     23     24     25     26     27     28                  ",
            "                                    ┌──────────────────────────                 ",
            "                  29     30     31  │   1      2      3      4   January        ",
            "                 ───────────────────┘                                           ",
            "           2025    5      6      7      8      9     10     11                  ",
            "                                                                                ",
            "                  12     13     14     15     16     17     18                  ",
            "                                                                                ",
            "                  19     20     21    [22]    23     24     25                  ",
            "                                                         ┌─────                 ",
            "                  26     27     28     29     30     31  │   1   February       ",
            "                 ────────────────────────────────────────┘                      ",
            "                   2      3      4      5      6      7      8                  ",
            "                                                                                ",
            "                   9     10     11     12     13     14     15                  ",
            "                                                                                ",
            "                  16     17     18     19     20     21     22                  ",
            "                                                         ┌─────                 ",
            "                  23     24     25     26     27     28  │   1   March          ",
            "                 ────────────────────────────────────────┘                      ",
        ]);
        expected.set_style(*expected.area(), BASE_STYLE);
        expected.set_style(Rect::new(17, 0, 46, 1), WEEKDAY_STYLE);
        expected.set_style(Rect::new(11, 2, 4, 1), YEAR_STYLE);
        expected.set_style(Rect::new(17, 2, 4, 1), FULL_MOON_STYLE);
        expected.set_style(Rect::new(24, 2, 4, 1), FULL_MOON_STYLE);
        expected.set_style(Rect::new(31, 2, 4, 1), FULL_MOON_STYLE);
        expected.set_style(Rect::new(65, 2, 8, 1), MONTH_STYLE);
        expected.set_style(Rect::new(17, 6, 4, 1), NEW_MOON_STYLE);
        expected.set_style(Rect::new(24, 6, 4, 1), NEW_MOON_STYLE);
        expected.set_style(Rect::new(31, 6, 4, 1), NEW_MOON_STYLE);
        expected.set_style(Rect::new(38, 6, 4, 1), NEW_MOON_STYLE);
        expected.set_style(Rect::new(45, 6, 4, 1), NEW_MOON_STYLE);
        expected.set_style(Rect::new(65, 6, 7, 1), MONTH_STYLE);
        expected.set_style(Rect::new(11, 8, 4, 1), YEAR_STYLE);
        expected.set_style(Rect::new(31, 10, 4, 1), FULL_MOON_STYLE);
        expected.set_style(Rect::new(38, 10, 4, 1), FULL_MOON_STYLE);
        expected.set_style(Rect::new(45, 10, 4, 1), FULL_MOON_STYLE);
        expected.set_style(Rect::new(52, 10, 4, 1), FULL_MOON_STYLE);
        expected.set_style(Rect::new(38, 14, 4, 1), NEW_MOON_STYLE);
        expected.set_style(Rect::new(45, 14, 4, 1), NEW_MOON_STYLE);
        expected.set_style(Rect::new(52, 14, 4, 1), NEW_MOON_STYLE);
        expected.set_style(Rect::new(59, 14, 4, 1), NEW_MOON_STYLE);
        expected.set_style(Rect::new(65, 14, 8, 1), MONTH_STYLE);
        expected.set_style(Rect::new(45, 18, 4, 1), FULL_MOON_STYLE);
        expected.set_style(Rect::new(52, 18, 4, 1), FULL_MOON_STYLE);
        expected.set_style(Rect::new(59, 18, 4, 1), FULL_MOON_STYLE);
        expected.set_style(Rect::new(45, 22, 4, 1), NEW_MOON_STYLE);
        expected.set_style(Rect::new(52, 22, 4, 1), NEW_MOON_STYLE);
        expected.set_style(Rect::new(59, 22, 4, 1), NEW_MOON_STYLE);
        expected.set_style(Rect::new(65, 22, 5, 1), MONTH_STYLE);
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_help() {
        let today = time::Date::from_calendar_date(2025, time::Month::January, 22).unwrap();
        let calpager = WeekWindow::new(today, Phoon);
        let mut app = App::new(calpager);
        app.handle_key(KeyCode::Char('?'));
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);
        app.render(area, &mut buffer);
        let mut expected = Buffer::with_lines([
            "                  Su     Mo     Tu     We     Th     Fr     Sa                  ",
            "                 ──────────────────────────────────────────────                 ",
            "           2024   15     16     17     18     19     20     21   December       ",
            "                                                                                ",
            "                  22     23     24     25     26     27     28                  ",
            "                                    ┌──────────────────────────                 ",
            "                  29 ┌───────────── Commands ──────────────┐ 4   January        ",
            "                 ─── │j, UP           Scroll up one week   │                    ",
            "           2025    5 │k, DOWN         Scroll down one week │ 1                  ",
            "                     │w, PAGE UP      Scroll up one page   │                    ",
            "                  12 │z, PAGE DOWN    Scroll down one page │ 8                  ",
            "                     │0, HOME         Jump to today        │                    ",
            "                  19 │g               Input date to jump to│ 5                  ",
            "                     │?               Show this help       │ ──                 ",
            "                  26 │q, ESC          Quit                 │ 1   February       ",
            "                 ─── │                                     │                    ",
            "                   2 │Press the Any Key to dismiss.        │ 8                  ",
            "                     └─────────────────────────────────────┘                    ",
            "                   9     10     11     12     13     14     15                  ",
            "                                                                                ",
            "                  16     17     18     19     20     21     22                  ",
            "                                                         ┌─────                 ",
            "                  23     24     25     26     27     28  │   1   March          ",
            "                 ────────────────────────────────────────┘                      ",
        ]);
        expected.set_style(*expected.area(), BASE_STYLE);
        expected.set_style(Rect::new(17, 0, 46, 1), WEEKDAY_STYLE);
        expected.set_style(Rect::new(11, 2, 4, 1), YEAR_STYLE);
        expected.set_style(Rect::new(17, 2, 4, 1), FULL_MOON_STYLE);
        expected.set_style(Rect::new(24, 2, 4, 1), FULL_MOON_STYLE);
        expected.set_style(Rect::new(31, 2, 4, 1), FULL_MOON_STYLE);
        expected.set_style(Rect::new(65, 2, 8, 1), MONTH_STYLE);
        expected.set_style(Rect::new(17, 6, 3, 1), NEW_MOON_STYLE);
        expected.set_style(Rect::new(65, 6, 7, 1), MONTH_STYLE);
        expected.set_style(Rect::new(11, 8, 4, 1), YEAR_STYLE);
        expected.set_style(Rect::new(61, 14, 2, 1), NEW_MOON_STYLE);
        expected.set_style(Rect::new(65, 14, 8, 1), MONTH_STYLE);
        expected.set_style(Rect::new(45, 18, 4, 1), FULL_MOON_STYLE);
        expected.set_style(Rect::new(52, 18, 4, 1), FULL_MOON_STYLE);
        expected.set_style(Rect::new(59, 18, 4, 1), FULL_MOON_STYLE);
        expected.set_style(Rect::new(45, 22, 4, 1), NEW_MOON_STYLE);
        expected.set_style(Rect::new(52, 22, 4, 1), NEW_MOON_STYLE);
        expected.set_style(Rect::new(59, 22, 4, 1), NEW_MOON_STYLE);
        expected.set_style(Rect::new(65, 22, 5, 1), MONTH_STYLE);
        assert_eq!(buffer, expected);
    }
}
