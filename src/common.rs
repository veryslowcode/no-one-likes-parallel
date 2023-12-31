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
    style::Color,
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
    Stopping,
    Switching(Screen, Option<PortParameters>),
}

#[derive(Debug, PartialEq)]
pub enum Message {
    Quit,
    Enter,
    Backspace,
    Input(char),
    NextElement,
    PreviousElement,
    Switching(Screen, Option<PortParameters>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Parity {
    Odd,
    Even,
    None,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Mode {
    Hex,
    Octal,
    Ascii,
    Decimal,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PortParameters {
    pub name: Option<String>,
    pub baud_rate: Option<u32>,
    pub data_bits: Option<u8>,
    pub stop_bits: Option<u8>,
    pub parity: Option<Parity>,
    pub mode: Option<Mode>,
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
* Implementation
*******************************************************************************/
/******************************************************************************/
impl Default for PortParameters {
    fn default() -> PortParameters {
        PortParameters {
            name: None,
            baud_rate: None,
            data_bits: None,
            stop_bits: None,
            parity: None,
            mode: None,
        }
    }
}

impl PortParameters {
    pub fn name(&mut self, n: String) -> Self {
        self.name = Some(n);
        return self.clone();
    }
}

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
