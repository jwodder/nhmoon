use crate::calendar::{Calendar, DateStyler, WeekWindow};
use crate::help::Help;
use crate::jumpto::{JumpTo, JumpToInput, JumpToOutput, JumpToState};
use crate::theme::BASE_STYLE;
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{backend::Backend, Frame, Terminal};
use std::io::{self, Write};

#[derive(Debug)]
pub(crate) struct App<S, B: Backend> {
    terminal: Terminal<B>,
    state: AppState<S>,
}

impl<S: DateStyler, B: Backend> App<S, B> {
    pub(crate) fn new(terminal: Terminal<B>, weeks: WeekWindow<S>) -> App<S, B> {
        App {
            terminal,
            state: AppState::new(weeks),
        }
    }

    pub(crate) fn run(mut self) -> io::Result<()> {
        while !self.state.quitting() {
            self.draw()?;
            self.handle_input()?;
        }
        Ok(())
    }

    fn draw(&mut self) -> io::Result<()> {
        self.terminal.draw(|frame| self.state.draw(frame))?;
        Ok(())
    }

    fn handle_input(&mut self) -> io::Result<()> {
        let normal_modifiers = KeyModifiers::NONE | KeyModifiers::SHIFT;
        if let Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            ..
        }) = read()?
        {
            if !normal_modifiers.contains(modifiers) || !self.handle_key(code) {
                self.beep()?;
            }
        }
        // else: Redraw on resize, and we might as well redraw on other stuff
        // too
        Ok(())
    }

    fn handle_key(&mut self, key: KeyCode) -> bool {
        self.state.handle_key(key)
    }

    fn beep(&self) -> io::Result<()> {
        io::stdout().write_all(b"\x07")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AppState<S> {
    weeks: WeekWindow<S>,
    inner: InnerAppState,
}

impl<S: DateStyler> AppState<S> {
    fn new(weeks: WeekWindow<S>) -> AppState<S> {
        AppState {
            weeks,
            inner: InnerAppState::Calendar,
        }
    }

    fn quitting(&self) -> bool {
        self.inner == InnerAppState::Quitting
    }

    fn draw(&mut self, frame: &mut Frame<'_>) {
        let size = frame.area();
        frame.buffer_mut().set_style(size, BASE_STYLE);
        let cal = Calendar::<S>::new();
        frame.render_stateful_widget(cal, size, &mut self.weeks);
        if self.inner == InnerAppState::Helping {
            frame.render_widget(Help, size);
        } else if let InnerAppState::Jumping(ref mut state) = self.inner {
            frame.render_stateful_widget(JumpTo, size, state);
        }
    }

    // Returns `false` if the user pressed an invalid key
    fn handle_key(&mut self, key: KeyCode) -> bool {
        match &mut self.inner {
            InnerAppState::Calendar => match key {
                KeyCode::Char('j') | KeyCode::Down => self.scroll_down(),
                KeyCode::Char('k') | KeyCode::Up => self.scroll_up(),
                KeyCode::Char('z') | KeyCode::PageDown => self.page_down(),
                KeyCode::Char('w') | KeyCode::PageUp => self.page_up(),
                KeyCode::Char('0') | KeyCode::Home => {
                    self.reset();
                    true
                }
                KeyCode::Char('g') => {
                    self.inner = InnerAppState::Jumping(JumpToState::new());
                    true
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    self.inner = InnerAppState::Quitting;
                    true
                }
                KeyCode::Char('?') => {
                    self.inner = InnerAppState::Helping;
                    true
                }
                _ => false,
            },
            InnerAppState::Helping => {
                self.inner = InnerAppState::Calendar;
                true
            }
            InnerAppState::Jumping(state) => {
                if matches!(key, KeyCode::Char('q' | 'g') | KeyCode::Esc) {
                    self.inner = InnerAppState::Calendar;
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
                            self.inner = InnerAppState::Calendar;
                            self.jump_to(date);
                            true
                        }
                    }
                }
            }
            InnerAppState::Quitting => false,
        }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InnerAppState {
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
    use ratatui::{backend::TestBackend, buffer::Buffer, layout::Rect};

    #[test]
    fn test_across_year() {
        let today = time::Date::from_calendar_date(2025, time::Month::January, 22).unwrap();
        let calpager = WeekWindow::new(today, Phoon);
        let mut app = App::new(Terminal::new(TestBackend::new(80, 24)).unwrap(), calpager);
        app.draw().unwrap();
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
        assert_eq!(app.terminal.backend().buffer(), &expected);
    }

    #[test]
    fn test_help() {
        let today = time::Date::from_calendar_date(2025, time::Month::January, 22).unwrap();
        let calpager = WeekWindow::new(today, Phoon);
        let mut app = App::new(Terminal::new(TestBackend::new(80, 24)).unwrap(), calpager);
        app.handle_key(KeyCode::Char('?'));
        app.draw().unwrap();
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
        assert_eq!(app.terminal.backend().buffer(), &expected);
    }
}
