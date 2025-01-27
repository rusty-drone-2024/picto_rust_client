mod chat_scroll_view;
mod draw_alert;
mod draw_chat_select;
mod draw_chat_view;
mod draw_help_box;
mod draw_name_set;
mod draw_room_select;
mod draw_text_edit;
pub(crate) mod ui_utils;

use crate::state::{ActiveComponent, TUIState};
use crate::ui::draw_alert::draw_alert;
use crate::ui::draw_chat_select::draw_chat_select;
use crate::ui::draw_chat_view::draw_chat_view;
use crate::ui::draw_help_box::draw_help_box;
use crate::ui::draw_name_set::draw_name_set;
use crate::ui::draw_room_select::draw_room_select;
use crate::ui::draw_text_edit::draw_text_edit;
use crate::ui::ui_utils::get_main_screen_rects;
use ratatui::prelude::{Style, Stylize};
use ratatui::widgets::Block;
use ratatui::Frame;
use std::cell::Ref;
use std::cmp::max;

// left main h split (name select & room select)
const ROOM_SELECT_MIN_HEIGHT: u16 = 10;
const NAME_SET_HEIGHT: u16 = 3;
const INFO_HEIGHT: u16 = 9;

const LEFT_MAIN_H_SPLIT_MIN_WIDTH: u16 = 32;
const LEFT_MAIN_H_SPLIT_MIN_HEIGHT: u16 = ROOM_SELECT_MIN_HEIGHT + NAME_SET_HEIGHT + INFO_HEIGHT;

// center main h split (chat select)
const CENTER_MAIN_H_SPLIT_MIN_WIDTH: u16 = 28;
const CENTER_MAIN_H_SPLIT_MIN_HEIGHT: u16 = 15;

// right main h split (chat view & message build)
const TEXT_EDIT_HEIGHT: u16 = 7;
const CHAT_MIN_HEIGHT: u16 = 20;

const RIGHT_MAIN_H_SPLIT_MIN_WIDTH: u16 = 42;
const RIGHT_MAIN_H_SPLIT_MIN_HEIGHT: u16 = TEXT_EDIT_HEIGHT + CHAT_MIN_HEIGHT;

//total
const MIN_WIDTH: u16 =
    LEFT_MAIN_H_SPLIT_MIN_WIDTH + CENTER_MAIN_H_SPLIT_MIN_WIDTH + RIGHT_MAIN_H_SPLIT_MIN_WIDTH;

pub fn ui(frame: &mut Frame, state: Ref<TUIState>) {
    let area = frame.area();
    frame.render_widget(Block::new().style(Style::new().white()), area);
    let mut min_height = max(LEFT_MAIN_H_SPLIT_MIN_HEIGHT, CENTER_MAIN_H_SPLIT_MIN_HEIGHT);
    min_height = max(min_height, RIGHT_MAIN_H_SPLIT_MIN_HEIGHT);
    if area.width < MIN_WIDTH || area.height < min_height {
        draw_alert(
            frame,
            frame.area(),
            "PictoRust needs a bigger window to display all its content. \nTry resizing!",
        );
    } else {
        match state.ui_data.active_component {
            ActiveComponent::Startup => {
                //startup_screen();
            }
            _ => {
                main_screen(frame, state);
            }
        }
    }
}

fn main_screen(frame: &mut Frame, state: Ref<TUIState>) {
    let (
        name_set_rect,
        room_select_rect,
        help_box_rect,
        chat_select_rect,
        chat_view_rect,
        text_edit_rect,
    ) = get_main_screen_rects(frame);

    draw_name_set(frame, name_set_rect, &state);
    draw_room_select(frame, room_select_rect, &state);
    draw_chat_select(frame, chat_select_rect, &state);
    draw_chat_view(frame, chat_view_rect, &state);
    draw_text_edit(frame, text_edit_rect, &state);
    draw_help_box(frame, help_box_rect, &state);
}
