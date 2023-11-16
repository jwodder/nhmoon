use crate::calpager::{CalPager, CalPagerWidget, DateStyler};
use crossterm::{
    event::{read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    style::Print,
};
use ratatui::prelude::*;
use std::io;

type CrossTerminal = Terminal<CrosstermBackend<io::Stdout>>;

#[derive(Debug)]
pub(crate) struct App<S> {
    terminal: CrossTerminal,
    calpager: CalPager<S>,
}

impl<S: DateStyler> App<S> {
    pub(crate) fn new(terminal: CrossTerminal, calpager: CalPager<S>) -> App<S> {
        App { terminal, calpager }
    }

    pub(crate) fn run(mut self) -> io::Result<()> {
        loop {
            self.terminal.draw(|frame| {
                let size = frame.size();
                frame
                    .buffer_mut()
                    .set_style(size, Style::default().white().on_black());
                let cpw = CalPagerWidget::<S>::new();
                frame.render_stateful_widget(cpw, size, &mut self.calpager);
            })?;
            match self.readkey()? {
                KeyCode::Esc => break,
                KeyCode::Char('j') => self.scroll_down(),
                KeyCode::Char('k') => self.scroll_up(),
                KeyCode::Char('z') => self.page_down(),
                KeyCode::Char('w') => self.page_up(),
                KeyCode::Char('0') => self.reset(),
                KeyCode::Char('q') => break,
                _ => self.beep()?,
            }
        }
        Ok(())
    }

    fn readkey(&mut self) -> io::Result<KeyCode> {
        let normal_modifiers = KeyModifiers::NONE | KeyModifiers::SHIFT;
        loop {
            if let Event::Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            }) = read()?
            {
                if normal_modifiers.contains(modifiers) {
                    return Ok(code);
                } else {
                    self.beep()?;
                }
            }
        }
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

    fn beep(&mut self) -> io::Result<()> {
        execute!(self.terminal.backend_mut(), Print("\x07"))
    }
}
