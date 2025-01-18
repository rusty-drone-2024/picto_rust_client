use crate::state::ActiveComponent::*;
use crate::state::{ActiveComponent, TUIState};
use crate::ui::draw_alert::draw_alert;
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::prelude::{Style, Text};
use ratatui::style::Stylize;
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, Cell, Paragraph, Row, Table};
use ratatui::Frame;
use std::cell::Ref;

pub(super) fn draw_chat_select(frame: &mut Frame, rect: Rect, state: &Ref<TUIState>) {
    let border_style = if let ChatSelect = state.ui_data.active_component {
        Style::new().green()
    } else {
        Style::new()
    };
    if let Some(r_id) = state.ui_data.current_room {
        let room = &state.chat_data.chat_rooms[r_id];
        let selected_log = if let Some(l_id) = state.ui_data.selected_log {
            Some(&room.chats[l_id])
        } else {
            None
        };

        let current_log = if let Some(l_id) = state.ui_data.current_log {
            Some(&room.chats[l_id])
        } else {
            None
        };

        let mut rows = Vec::new();
        for log in &room.chats {
            let peer_name = log.peer_name.clone();
            let last_seen = log.last_seen.to_string();
            let online = if log.net_reachable { '🌐' } else { '❌' };
            let unread = if log.pending > 0 {
                log.pending.to_string()
            } else {
                '✔'.to_string()
            };
            let mut row_style = Style::new();
            let mut pending_style = if log.pending > 0 {
                Style::new().yellow()
            } else {
                Style::new().green()
            };
            let mut online_style = if log.net_reachable {
                Style::new().on_green()
            } else {
                Style::new().on_red()
            };

            if let Some(l) = current_log {
                if l.id == log.id {
                    row_style = Style::new().black().on_green();
                    pending_style = row_style.clone();
                    online_style = Style::new();
                }
            }

            if let Some(l) = selected_log {
                if l.id == log.id {
                    row_style = Style::new().black().on_gray();
                    pending_style = row_style.clone();
                    online_style = Style::new();
                }
            }

            rows.push(
                Row::new(vec![
                    Cell::new(Text::from(peer_name)),
                    Cell::new(Text::from(last_seen).alignment(Alignment::Center)),
                    Cell::new(
                        Text::from(online.to_string())
                            .alignment(Alignment::Center)
                            .style(online_style),
                    ),
                    Cell::new(
                        Text::from(unread.to_string())
                            .alignment(Alignment::Right)
                            .style(pending_style),
                    ),
                ])
                .style(row_style),
            );
        }

        let widths = [
            Constraint::Min(12),
            Constraint::Min(11),
            Constraint::Length(7),
            Constraint::Length(6),
        ];

        let center_alignment = if rect.width < 38 {
            Alignment::Left
        } else {
            Alignment::Center
        };

        let right_alignment = if rect.width < 38 {
            Alignment::Left
        } else {
            Alignment::Right
        };

        let header_style_1 = if let ChatSelect = &state.ui_data.active_component {
            Style::new().black().on_green()
        } else {
            Style::new().on_black()
        };
        let header_style_2 = if let ChatSelect = &state.ui_data.active_component {
            Style::new().black().on_light_green()
        } else {
            Style::new()
        };

        let table = Table::new(rows, widths).header(Row::new([
            Cell::new(Text::from("Name ")).style(header_style_2),
            Cell::new(Text::from("Last seen ").alignment(center_alignment)).style(header_style_1),
            Cell::new(Text::from("Online ").alignment(center_alignment)).style(header_style_2),
            Cell::new(Text::from("Unread").alignment(right_alignment)).style(header_style_1),
        ]));

        frame.render_widget(
            table
                .block(
                    Block::bordered()
                        .border_type(Rounded)
                        .border_style(border_style)
                        .title(format!("Room {} friends ({})", room.id, room.chats.len())),
                )
                .style(Style::new())
                .column_spacing(0),
            rect,
        );
    } else {
        let block = Block::bordered()
            .border_type(Rounded)
            .border_style(border_style)
            .title("Chat select")
            .style(Style::new());

        frame.render_widget(block, rect);
        draw_alert(frame, rect, "Select a room first!");
    }
}
