use crate::state::ActiveComponent::NameSet;
use crate::state::NameSetAction::*;
use crate::state::TUIState;
use ratatui::layout::Rect;
use ratatui::prelude::Style;
use ratatui::style::Stylize;
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;
use std::cell::Ref;

pub(crate) fn draw_name_set(frame: &mut Frame, rect: Rect, state: &Ref<TUIState>) {
    let active_component = &state.ui_data.active_component;
    let border_style = if let NameSet(_) = active_component {
        Style::new().green()
    } else {
        Style::new()
    };

    match active_component {
        NameSet(ChangingName) => {
            if let Some(name) = &state.ui_data.name_in_editing {
                frame.render_widget(
                    Paragraph::new(name.clone())
                        .block(
                            Block::bordered()
                                .border_type(Rounded)
                                .title("Change name")
                                .border_style(border_style),
                        )
                        .style(Style::new()),
                    rect,
                );
            }
        }
        _ => {
            frame.render_widget(
                Paragraph::new(state.chat_data.current_name.clone())
                    .block(
                        Block::bordered()
                            .border_type(Rounded)
                            .title("Name")
                            .border_style(border_style),
                    )
                    .style(Style::new()),
                rect,
            );
        }
    }
}
