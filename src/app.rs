use crate::calendar::{Calendar, DateStyler, WeekWindow};
use crate::help::Help;
use crossterm::{
    event::{read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    style::Print,
};
use ratatui::{
    style::{Style, Stylize},
    DefaultTerminal, Frame,
};
use std::io;

#[derive(Debug)]
pub(crate) struct App<S> {
    terminal: DefaultTerminal,
    state: AppState<S>,
}

impl<S: DateStyler> App<S> {
    pub(crate) fn new(terminal: DefaultTerminal, weeks: WeekWindow<S>) -> App<S> {
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
            if !normal_modifiers.contains(modifiers) || !self.state.handle_key(code) {
                self.beep()?;
            }
        }
        // else: Redraw on resize, and we might as well redraw on other stuff
        // too
        Ok(())
    }

    fn beep(&mut self) -> io::Result<()> {
        execute!(self.terminal.backend_mut(), Print("\x07"))
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
        let defstyle = Style::default().white().on_black();
        frame.buffer_mut().set_style(size, defstyle);
        let cal = Calendar::<S>::new();
        frame.render_stateful_widget(cal, size, &mut self.weeks);
        if self.inner == InnerAppState::Helping {
            frame.render_widget(Help(defstyle), size);
        }
    }

    // Returns `false` if the user pressed an invalid key
    fn handle_key(&mut self, key: KeyCode) -> bool {
        match self.inner {
            InnerAppState::Calendar => match key {
                KeyCode::Char('j') | KeyCode::Down => self.scroll_down(),
                KeyCode::Char('k') | KeyCode::Up => self.scroll_up(),
                KeyCode::Char('z') | KeyCode::PageDown => self.page_down(),
                KeyCode::Char('w') | KeyCode::PageUp => self.page_up(),
                KeyCode::Char('0') | KeyCode::Home => {
                    self.reset();
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InnerAppState {
    Calendar,
    Helping,
    Quitting,
}
