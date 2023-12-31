/*******************************************************************************/
/********************************************************************************
* DESCRIPTION: Defines shared interfaces, behavior, and constants.
* AUTHOR: jb
* DATE: 12/30/23
********************************************************************************/
/*******************************************************************************/
use ratatui::Frame;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
};

/******************************************************************************/
/*******************************************************************************
* Public Interface
*******************************************************************************/
/******************************************************************************/
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Screen {
    #[default]
    Menu,
    DeviceList,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum State {
    #[default]
    Running,
    Switching(Screen),
    Stopping,
}

#[derive(Debug, PartialEq)]
pub enum Message {
    Enter,
    Backspace,
    Input(char),
    NextElement,
    PreviousElement,
    Switching(Screen),
    Quit,
}

pub trait Tea {
    fn update(&mut self, msg: Message) -> State;
    fn view(&mut self, f: &mut Frame);
}

pub trait Nolp {
    fn get_state(&self) -> State;
    fn set_state(&mut self, s: State);
}

/******************************************************************************/
/*******************************************************************************
* Global Constants
*******************************************************************************/
/******************************************************************************/
pub const INVALID_COLOR: Color = Color::LightRed;
pub const SELECTED_COLOR: Color = Color::LightBlue;
pub const PLACEHOLDER_COLOR: Color = Color::DarkGray;

/******************************************************************************/
/*******************************************************************************
* Utility Functions
*******************************************************************************/
/******************************************************************************/
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
