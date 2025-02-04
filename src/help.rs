use ratatui::{
    buffer::Buffer,
    layout::Flex,
    layout::{Alignment, Layout, Rect},
    style::Style,
    text::{Line, Text},
    widgets::{Block, Clear, Paragraph, Widget},
};

static TEXT: &[&str] = &[
    "j, UP           Scroll up one week\n",
    "k, DOWN         Scroll down one week\n",
    "w, PAGE UP      Scroll up one page\n",
    "z, PAGE DOWN    Scroll down one page\n",
    "0, HOME         Jump to today\n",
    "?               Show this help\n",
    "q, ESC          Quit\n",
    "\n",
    "Press the Any Key to dismiss.\n",
];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Help(pub(crate) Style);

impl Widget for Help {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lines = TEXT.iter().map(|&s| Line::raw(s)).collect::<Vec<_>>();
        let text = Text::from(lines);
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
                    .title_alignment(Alignment::Center),
            )
            .style(self.0);
        let [help_area] = Layout::horizontal([width]).flex(Flex::Center).areas(area);
        let [help_area] = Layout::vertical([height])
            .flex(Flex::Center)
            .areas(help_area);
        let outer_area = Rect {
            x: help_area.x.saturating_sub(1),
            y: help_area.y,
            width: help_area.width.saturating_add(2),
            height: help_area.height,
        };
        Clear.render(outer_area, buf);
        Block::new().style(self.0).render(outer_area, buf);
        para.render(help_area, buf);
    }
}
