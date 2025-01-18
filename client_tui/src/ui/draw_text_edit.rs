use crate::state::ActiveComponent::TextEdit;
use crate::state::TUIState;
use crate::state::TextEditAction::*;
use crate::ui::draw_alert::draw_alert;
use ratatui::layout::Constraint::{Fill, Length};
use ratatui::layout::Direction::{Horizontal, Vertical};
use ratatui::layout::{Layout, Rect};
use ratatui::prelude::Style;
use ratatui::style::Stylize;
use ratatui::widgets::BorderType::{QuadrantInside, Rounded};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use std::cell::Ref;

pub(super) fn draw_text_edit(frame: &mut Frame, rect: Rect, state: &Ref<TUIState>) {
    if let Some(l_id) = state.ui_data.current_log {
        let editor_rect = Rect::new(rect.x, rect.y, 42, rect.height);
        let send_button_rect = Rect::new(rect.x + 42, rect.y, rect.width - 42, rect.height);
        draw_text_area(frame, editor_rect, state);
        draw_send_button(frame, send_button_rect, state);
    } else {
        let border_style = if let TextEdit(_) = state.ui_data.active_component {
            Style::new().green()
        } else {
            Style::new()
        };
        frame.render_widget(
            Block::bordered()
                .border_type(Rounded)
                .border_style(border_style)
                .style(Style::new()),
            rect,
        );
        draw_alert(frame, rect, "Select a friend first!");
    }
}

fn draw_text_area(frame: &mut Frame, rect: Rect, state: &Ref<TUIState>) {
    let border_style = match state.ui_data.active_component {
        TextEdit(Editing) => Style::new().green(),
        _ => Style::new(),
    };

    let text = state.ui_data.text_message_in_edit.clone();
    let text_len = text.len();

    let editor = Paragraph::new(text).block(
        Block::new()
            .border_type(Rounded)
            .border_style(border_style)
            .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
            .title(format!("Write something: {}/200", text_len)),
    );
    frame.render_widget(editor, rect);
}

fn draw_send_button(frame: &mut Frame, rect: Rect, state: &Ref<TUIState>) {
    let border_style = match state.ui_data.active_component {
        TextEdit(ReadyToSend) => Style::new().green(),
        _ => Style::new(),
    };
    let button_inner_style = match state.ui_data.active_component {
        TextEdit(ReadyToSend) => Style::new().dark_gray().on_green(),
        _ => Style::new().dark_gray().on_white(),
    };
    let send_button_rect_h_split = Layout::default()
        .direction(Horizontal)
        .constraints([Fill(1), Length(4), Fill(1)])
        .split(rect);
    let send_button_rect_v_split = Layout::default()
        .direction(Vertical)
        .constraints([Fill(1), Length(1), Fill(1)])
        .split(send_button_rect_h_split[1]);
    let send_button_paragraph = Paragraph::new("Send").style(button_inner_style);

    let send_button = Block::bordered()
        .border_style(border_style)
        .border_type(QuadrantInside);
    let send_button_inner = Block::new().style(button_inner_style);
    let send_button_inner_rect = Rect::new(rect.x + 1, rect.y + 1, rect.width - 2, rect.height - 2);
    frame.render_widget(send_button, rect);
    frame.render_widget(send_button_inner, send_button_inner_rect);
    frame.render_widget(send_button_paragraph, send_button_rect_v_split[1]);
}
