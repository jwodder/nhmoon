use ratatui::style::{Color, Modifier, Style};

pub(crate) const BASE_STYLE: Style = Style::new().fg(Color::White).bg(Color::Black);

pub(crate) const FULL_MOON_STYLE: Style = Style::new()
    .fg(Color::LightYellow)
    .bg(Color::Black)
    .add_modifier(Modifier::BOLD);

pub(crate) const NEW_MOON_STYLE: Style = Style::new().fg(Color::LightBlue).bg(Color::Black);

pub(crate) const YEAR_STYLE: Style = BASE_STYLE.add_modifier(Modifier::BOLD);

pub(crate) const MONTH_STYLE: Style = BASE_STYLE.add_modifier(Modifier::BOLD);

pub(crate) const WEEKDAY_STYLE: Style = BASE_STYLE.add_modifier(Modifier::BOLD);

pub(crate) mod jumpto {
    use super::*;

    pub(crate) const UNFILLED_CELL_STYLE: Style = BASE_STYLE.fg(Color::DarkGray);

    pub(crate) const READY_ENTER_STYLE: Style = BASE_STYLE.add_modifier(Modifier::UNDERLINED);
}
