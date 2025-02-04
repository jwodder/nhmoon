use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Flex, Layout, Margin, Rect},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Clear, StatefulWidget, Widget},
};
use std::borrow::Cow;

const OUTER_WIDTH: u16 = 17;
const OUTER_HEIGHT: u16 = 8;
const ENTER_POS: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct JumpTo(pub(crate) Style);

impl StatefulWidget for JumpTo {
    type State = JumpToState;

    /*
     * .................
     * .┌─ Jump To… ──┐.
     * .│             │.
     * .│ -YYYY-MM-DD │.
     * .│             │.
     * .│   [ENTER]   │.
     * .└─────────────┘.
     * .................
     */

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let [outer_area] = Layout::horizontal([OUTER_WIDTH])
            .flex(Flex::Center)
            .areas(area);
        let [outer_area] = Layout::vertical([OUTER_HEIGHT])
            .flex(Flex::Center)
            .areas(outer_area);
        Clear.render(outer_area, buf);
        Block::new().style(self.0).render(outer_area, buf);
        let block_area = outer_area.inner(Margin::new(1, 1));
        Block::bordered()
            .title(" Jump To… ")
            .title_alignment(Alignment::Center)
            .render(block_area, buf);
        let text_area = block_area.inner(Margin::new(1, 1));
        state.to_text(self.0).render(text_area, buf);
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct JumpToState {
    negative: bool,
    year: [Option<u8>; 4],
    month: [Option<u8>; 2],
    day: [Option<u8>; 2],
    pos: usize,
}

impl JumpToState {
    pub(crate) fn new() -> JumpToState {
        JumpToState::default()
    }

    fn to_text(self, style: Style) -> Text<'static> {
        Text::from_iter([
            Line::styled("", style),
            self.to_line(style),
            Line::styled("", style),
            // Style a span and convert it to a line rather than creating a
            // styled line directly so that only the "[ENTER]" text and not any
            // of its centering padding will be underlined:
            Line::from(Span::styled(
                "[ENTER]",
                if self.pos == ENTER_POS {
                    style.underlined()
                } else {
                    style
                },
            )),
        ])
        .centered()
    }

    fn to_line(self, style: Style) -> Line<'static> {
        let mut spans = Vec::new();
        spans.push(Span::styled(if self.negative { "-" } else { " " }, style));
        let mut first = true;
        let mut i = 0;
        for (fallback, digits) in [
            ("Y", self.year.as_slice()),
            ("M", self.month.as_slice()),
            ("D", self.day.as_slice()),
        ] {
            if !std::mem::replace(&mut first, false) {
                spans.push(Span::styled("-", style));
            }
            for dg in digits {
                let s = match dg {
                    Some(d) => Cow::from(format!("{d}")),
                    None => Cow::from(fallback),
                };
                spans.push(Span::styled(
                    s,
                    if i == self.pos {
                        style.underlined()
                    } else {
                        style
                    },
                ));
                i += 1;
            }
        }
        Line::from_iter(spans)
    }

    pub(crate) fn handle_input(&mut self, input: JumpToInput) -> JumpToOutput {
        match (input, self.pos) {
            (JumpToInput::Negative, 0) => {
                self.negative = !self.negative;
                JumpToOutput::Ok
            }
            (JumpToInput::Positive, 0) => {
                self.negative = false;
                JumpToOutput::Ok
            }
            (JumpToInput::Digit(d), 0..ENTER_POS) => {
                match self.pos {
                    0..4 => self.year[self.pos] = Some(d),
                    4..6 => self.month[self.pos - 4] = Some(d),
                    6..8 => self.day[self.pos - 6] = Some(d),
                    _ => unreachable!(),
                }
                self.pos += 1;
                JumpToOutput::Ok
            }
            (JumpToInput::Backspace, 1..) => {
                self.pos -= 1;
                match self.pos {
                    0..4 => self.year[self.pos] = None,
                    4..6 => self.month[self.pos - 4] = None,
                    6..8 => self.day[self.pos - 6] = None,
                    _ => unreachable!(),
                }
                JumpToOutput::Ok
            }
            (JumpToInput::Enter, ENTER_POS) => {
                let mut year = 0i32;
                for d in self.year {
                    let d = d.expect("All year digits should be set");
                    year = year * 10 + i32::from(d);
                }
                if self.negative {
                    year *= -1;
                }
                let mut month = 0u8;
                for d in self.month {
                    let d = d.expect("All month digits should be set");
                    month = month * 10 + d;
                }
                let Ok(month) = time::Month::try_from(month) else {
                    return JumpToOutput::Invalid;
                };
                let mut day = 0u8;
                for d in self.month {
                    let d = d.expect("All day digits should be set");
                    day = day * 10 + d;
                }
                match time::Date::from_calendar_date(year, month, day) {
                    Ok(date) => JumpToOutput::Jump(date),
                    Err(_) => JumpToOutput::Invalid,
                }
            }
            _ => JumpToOutput::Invalid,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum JumpToInput {
    Negative,
    Positive,
    Digit(u8),
    Backspace,
    Enter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum JumpToOutput {
    Ok,
    Invalid,
    Jump(time::Date),
}
