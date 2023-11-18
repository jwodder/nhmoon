use crate::calpager::{CalPager, CalPagerWidget, DateStyler};
use crate::help::Help;
use crossterm::{
    event::{read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    style::Print,
};
use ratatui::prelude::*;
use std::io;

pub(crate) type CrossTerminal = Terminal<CrosstermBackend<io::Stdout>>;

#[derive(Debug)]
pub(crate) struct App<S> {
    terminal: CrossTerminal,
    calpager: CalPager<S>,
    quitting: bool,
    helping: bool,
}

impl<S: DateStyler> App<S> {
    pub(crate) fn new(terminal: CrossTerminal, calpager: CalPager<S>) -> App<S> {
        App {
            terminal,
            calpager,
            quitting: false,
            helping: false,
        }
    }

    pub(crate) fn run(mut self) -> io::Result<()> {
        while !self.quitting {
            self.draw()?;
            self.handle_input()?;
        }
        Ok(())
    }

    fn draw(&mut self) -> io::Result<()> {
        self.terminal.draw(|frame| {
            let size = frame.size();
            let defstyle = Style::default().white().on_black();
            frame.buffer_mut().set_style(size, defstyle);
            let cpw = CalPagerWidget::<S>::new();
            frame.render_stateful_widget(cpw, size, &mut self.calpager);
            if self.helping {
                frame.render_widget(Help(defstyle), size);
            }
        })?;
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
            if normal_modifiers.contains(modifiers) {
                self.handle_key(code)?;
            } else {
                self.beep()?;
            }
        }
        // else: Redraw on resize, and we might as well redraw on other stuff
        // too
        Ok(())
    }

    fn handle_key(&mut self, key: KeyCode) -> io::Result<()> {
        if self.helping {
            self.helping = false;
            return Ok(());
        }
        match key {
            KeyCode::Char('j') | KeyCode::Down => self.scroll_down(),
            KeyCode::Char('k') | KeyCode::Up => self.scroll_up(),
            KeyCode::Char('z') | KeyCode::PageDown => self.page_down(),
            KeyCode::Char('w') | KeyCode::PageUp => self.page_up(),
            KeyCode::Char('0') | KeyCode::Home => self.reset(),
            KeyCode::Char('q') | KeyCode::Esc => self.quit(),
            KeyCode::Char('?') => self.helping = true,
            _ => self.beep()?,
        }
        Ok(())
    }

    fn scroll_down(&mut self) {
        self.calpager.one_week_forwards();
    }

    fn scroll_up(&mut self) {
        self.calpager.one_week_backwards();
    }

    fn page_down(&mut self) {
        self.calpager.one_page_forwards();
    }

    fn page_up(&mut self) {
        self.calpager.one_page_backwards();
    }

    fn reset(&mut self) {
        self.calpager.jump_to_today();
    }

    fn quit(&mut self) {
        self.quitting = true;
    }

    fn beep(&mut self) -> io::Result<()> {
        execute!(self.terminal.backend_mut(), Print("\x07"))
    }
}
