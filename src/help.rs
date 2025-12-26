use crate::theme::BASE_STYLE;
use ratatui::{
    buffer::Buffer,
    layout::{Flex, HorizontalAlignment, Layout, Rect},
    text::Text,
    widgets::{Block, Clear, Paragraph, Widget},
};

static TEXT: &[&str] = &[
    "j, UP           Scroll up one week",
    "k, DOWN         Scroll down one week",
    "w, PAGE UP      Scroll up one page",
    "z, PAGE DOWN    Scroll down one page",
    "0, HOME         Jump to today",
    "g               Input date to jump to",
    "?               Show this help",
    "q, ESC          Quit",
    "",
    "Press the Any Key to dismiss.",
];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Help;

impl Widget for Help {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = Text::from_iter(TEXT.iter().copied());
        let height = u16::try_from(text.height())
            .unwrap_or(u16::MAX)
            .min(area.height)
            .saturating_add(2);
        let width = u16::try_from(text.width())
            .unwrap_or(u16::MAX)
            .min(area.width)
            .saturating_add(2);
        let para = Paragraph::new(text)
            .block(
                Block::bordered()
                    .title(" Commands ")
                    .title_alignment(HorizontalAlignment::Center),
            )
            .style(BASE_STYLE);
        let [help_area] = Layout::horizontal([width]).flex(Flex::Center).areas(area);
        let [help_area] = Layout::vertical([height])
            .flex(Flex::Center)
            .areas(help_area);
        let outer_area = Rect {
            x: help_area.x.saturating_sub(1),
            y: help_area.y,
            width: help_area.width.saturating_add(2).min(area.width),
            height: help_area.height,
        };
        Clear.render(outer_area, buf);
        Block::new().style(BASE_STYLE).render(outer_area, buf);
        para.render(help_area, buf);
    }
}
