use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

#[derive(Debug, Default, Eq, PartialEq)]
pub enum State {
    #[default]
    Running,
    Switching,
    Stopping,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Message {
    NextElement,
    PreviousElement,
    Quit,
}

pub trait Tea {
    fn update(&mut self, msg: Message);
    fn view(&mut self, f: &mut Frame);
}

pub trait Nolp {
    fn get_state(&mut self) -> &State;
    fn set_state(&mut self, s: State);
}

pub fn get_center_bounds(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let v_constraint = (100 - percent_y) / 2;
    let h_constraint = (100 - percent_x) / 2;
    let v_center = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(v_constraint),
            Constraint::Percentage(percent_y),
            Constraint::Percentage(v_constraint),
        ])
        .split(area);
    let h_center = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(h_constraint),
            Constraint::Percentage(percent_x),
            Constraint::Percentage(h_constraint),
        ])
        .split(v_center[1]);

    return h_center[1];
}
