use crate::state::ChatMessage;
use client_lib::communication::MessageContent::*;
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint::{Fill, Length};
use ratatui::layout::{Layout, Position, Rect, Size};
use ratatui::prelude::StatefulWidget;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, Paragraph, Widget, Wrap};
use std::cell::RefMut;
use tui_scrollview::ScrollbarVisibility::{Always, Never};
use tui_scrollview::{ScrollView, ScrollViewState};
use unicode_width::UnicodeWidthStr;

pub(crate) struct ChatScrollView<'a> {
    pub(crate) messages: &'a Vec<ChatMessage>,
    pub(crate) scroll_view_state: RefMut<'a, ScrollViewState>,
    pub(crate) go_to_chat_bottom: RefMut<'a, bool>,
    pub(crate) selected_message: RefMut<'a, Option<usize>>,
}

impl<'a> ChatScrollView<'a> {
    pub(crate) fn render_messages_into_scrollview(&self, buf: &mut Buffer) {
        let area = buf.area;
        let mut current_height = 0;
        let line_w = area.width - 2;
        for (i, m) in self.messages.iter().enumerate() {
            let p = self.message(m, i);
            let msg_w = Self::get_msg_width(m, line_w);
            let h = p.line_count(msg_w - 2) as u16;
            let rect = Rect::new(area.x, current_height, line_w, h);

            if m.status.is_some() {
                let msg_rect = Layout::horizontal([Fill(1), Length(msg_w)]).areas::<2>(rect)[1];
                p.render(msg_rect, buf);
            } else {
                let msg_rect = Layout::horizontal([Length(msg_w), Fill(1)]).areas::<2>(rect)[0];
                p.render(msg_rect, buf);
            }

            current_height += h;
        }
    }

    fn get_msg_width(m: &ChatMessage, w: u16) -> u16 {
        let mut msg_w = (w * 80) / 100;
        if let Some(c) = &m.content {
            if let TextMessage(s) = c {
                let len = UnicodeWidthStr::width(s.as_str());
                if len + 2 < msg_w as usize {
                    msg_w = (len + 2) as u16;
                }
            }
        } else {
            msg_w = 21;
        }
        msg_w
    }

    fn get_height(&self, w: u16) -> u16 {
        let mut current_height = 0;
        let line_w = w - 2;
        for (i, m) in self.messages.iter().enumerate() {
            let p = self.message(m, i);
            let msg_w = Self::get_msg_width(m, line_w);
            let h = p.line_count(msg_w - 2) as u16;
            current_height += h;
        }
        current_height
    }

    fn message(&self, m: &ChatMessage, id: usize) -> Paragraph {
        let mut border_style = Style::default();
        if let Some(sm) = *self.selected_message {
            if sm == id {
                border_style = Style::new().green();
            }
        }
        let mc = &m.content;
        if let Some(mci) = mc {
            return if let TextMessage(s) = mci {
                Paragraph::new(s.clone())
                    .block(
                        Block::bordered()
                            .border_type(Rounded)
                            .border_style(border_style),
                    )
                    .wrap(Wrap { trim: false })
            } else {
                Paragraph::new("Disegno")
                    .block(
                        Block::bordered()
                            .border_type(Rounded)
                            .border_style(border_style),
                    )
                    .wrap(Wrap { trim: false })
            };
        }
        Paragraph::new("Messaggio eliminato")
            .block(
                Block::bordered()
                    .border_type(Rounded)
                    .border_style(border_style),
            )
            .wrap(Wrap { trim: false })
    }
}

impl<'a> Widget for &mut ChatScrollView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let h = self.get_height(area.width - 2);
        let mut scroll_view = ScrollView::new(Size::new(area.width, h))
            .horizontal_scrollbar_visibility(Never)
            .vertical_scrollbar_visibility(Always);
        if *self.go_to_chat_bottom && area.height < h {
            self.scroll_view_state.set_offset(Position {
                x: 0,
                y: h - area.height,
            });
            *self.go_to_chat_bottom = false;
        }
        self.render_messages_into_scrollview(scroll_view.buf_mut());
        scroll_view.render(area, buf, &mut self.scroll_view_state)
    }
}
