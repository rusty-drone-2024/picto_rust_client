use crate::ui::{
    CENTER_MAIN_H_SPLIT_MIN_WIDTH, INFO_HEIGHT, LEFT_MAIN_H_SPLIT_MIN_WIDTH, NAME_SET_HEIGHT,
    RIGHT_MAIN_H_SPLIT_MIN_WIDTH, TEXT_EDIT_HEIGHT,
};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

pub(super) fn centered_percent_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let v_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(v_split[1])[1] // Return the middle chunk
}

pub(crate) fn get_main_screen_rects(frame: &Frame) -> (Rect, Rect, Rect, Rect, Rect, Rect) {
    let area = frame.area();
    let main_h_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(LEFT_MAIN_H_SPLIT_MIN_WIDTH),
            Constraint::Min(CENTER_MAIN_H_SPLIT_MIN_WIDTH),
            Constraint::Min(RIGHT_MAIN_H_SPLIT_MIN_WIDTH),
        ])
        .split(area);
    let left_v_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(NAME_SET_HEIGHT),
            Constraint::Fill(1),
            Constraint::Length(INFO_HEIGHT),
        ])
        .split(main_h_split[0]);
    let right_v_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(TEXT_EDIT_HEIGHT)])
        .split(main_h_split[2]);

    (
        left_v_split[0],
        left_v_split[1],
        left_v_split[2],
        main_h_split[1],
        right_v_split[0],
        right_v_split[1],
    )
}
