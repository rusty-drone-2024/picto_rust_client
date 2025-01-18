use crate::state::ActiveComponent::*;
use crate::state::{ActiveComponent, TUIState};
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::prelude::Style;
use ratatui::style::{Styled, Stylize};
use ratatui::text::Text;
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, Cell, Row, Table};
use ratatui::Frame;
use std::cell::Ref;

pub(super) fn draw_room_select(frame: &mut Frame, rect: Rect, state: &Ref<TUIState>) {
    let border_style = if let ActiveComponent::RoomSelect = state.ui_data.active_component {
        Style::new().green()
    } else {
        Style::new()
    };

    let selected_room = if let Some(r_id) = state.ui_data.selected_room {
        Some(&state.chat_data.chat_rooms[r_id])
    } else {
        None
    };

    let current_room = if let Some(r_id) = state.ui_data.current_room {
        Some(&state.chat_data.chat_rooms[r_id])
    } else {
        None
    };

    let mut rows = Vec::new();
    for room in &state.chat_data.chat_rooms {
        let connected = if room.net_reachable { '🌐' } else { '❌' };
        let registered = if room.registered_to { '🤝' } else { '🫱' };
        let friends = if room.registered_to {
            room.chats.len().to_string()
        } else {
            "".to_string()
        };
        let mut unread = if room.pending > 0 {
            room.pending.to_string()
        } else {
            '✔'.to_string()
        };
        let mut pending_style = if room.pending > 0 {
            Style::new().yellow()
        } else {
            Style::new().green()
        };
        let mut row_style = Style::new();
        let mut online_style = if room.net_reachable {
            Style::new().on_green()
        } else {
            Style::new().on_red()
        };
        let mut registered_style = if room.registered_to {
            Style::new().on_green()
        } else {
            unread = "".to_string();
            Style::new().on_red()
        };

        if let Some(r) = current_room {
            if r.id == room.id {
                row_style = Style::new().black().on_green();
                pending_style = row_style;
                online_style = Style::new();
                registered_style = Style::new();
            }
        }

        if let Some(r) = selected_room {
            if r.id == room.id {
                row_style = Style::new().black().on_gray();
                pending_style = row_style;
                online_style = Style::new();
                registered_style = Style::new();
            }
        }

        rows.push(
            Row::new(vec![
                Cell::new(Text::from(room.id.to_string())),
                Cell::new(
                    Text::from(connected.to_string())
                        .alignment(Alignment::Center)
                        .style(online_style),
                ),
                Cell::new(
                    Text::from(registered.to_string())
                        .alignment(Alignment::Center)
                        .style(registered_style),
                ),
                Cell::new(Text::from(friends).alignment(Alignment::Center)),
                Cell::new(
                    Text::from(unread)
                        .alignment(Alignment::Right)
                        .style(pending_style),
                ),
            ])
            .style(row_style),
        );
    }

    let widths = [
        Constraint::Min(5),
        Constraint::Length(7),
        Constraint::Length(11),
        Constraint::Length(8),
        Constraint::Length(6),
    ];

    let center_alignment = if rect.width < 39 {
        Alignment::Left
    } else {
        Alignment::Center
    };

    let right_alignment = if rect.width < 39 {
        Alignment::Left
    } else {
        Alignment::Right
    };

    let header_style_1 = if let RoomSelect = state.ui_data.active_component {
        Style::new().black().on_green()
    } else {
        Style::new().on_black()
    };
    let header_style_2 = if let RoomSelect = state.ui_data.active_component {
        Style::new().black().on_light_green()
    } else {
        Style::new()
    };

    let table = Table::new(rows, widths).header(Row::new([
        Cell::new(Text::from("Room ")).style(header_style_2),
        Cell::new(Text::from("Online ").alignment(center_alignment)).style(header_style_1),
        Cell::new(Text::from("Registered ").alignment(center_alignment)).style(header_style_2),
        Cell::new(Text::from("Friends ").alignment(center_alignment)).style(header_style_1),
        Cell::new(Text::from("Unread").alignment(right_alignment)).style(header_style_2),
    ]));

    frame.render_widget(
        table
            .block(
                Block::bordered()
                    .border_type(Rounded)
                    .border_style(border_style)
                    .title(format!("Rooms ({})", state.chat_data.chat_rooms.len())),
            )
            .column_spacing(0)
            .style(Style::new()),
        rect,
    );
}
