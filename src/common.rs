/*******************************************************************************/
/********************************************************************************
* DESCRIPTION: Defines shared interfaces, behavior, and constants.
* AUTHOR: jb
* DATE: 12/30/23
********************************************************************************/
/*******************************************************************************/
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
    Frame,
};
use std::sync::{Arc, Mutex};

/******************************************************************************/
/*******************************************************************************
* Public Interface
*******************************************************************************/
/******************************************************************************/
pub type SerialFlag = Arc<Mutex<bool>>;
pub type SerialBuffer = Arc<Mutex<Vec<u8>>>;
pub type SerialError = Arc<Mutex<Option<String>>>;
pub type SerialParams = Arc<Mutex<PortParameters>>;

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Screen {
    #[default]
    Menu,
    Help,
    Terminal,
    DeviceList,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum State {
    #[default]
    Running,
    Pausing,
    Stopping,
    Error(String),
    Switching(Screen, Option<PortParameters>),
}

#[derive(Debug, PartialEq)]
pub enum Message {
    Quit,
    Enter,
    Pause,
    Resume,
    Rx(Vec<u8>),
    Backspace,
    Input(char),
    NextElement,
    PreviousElement,
    Switching(Screen, Option<PortParameters>),
}

#[derive(Debug, PartialEq)]
pub enum NolpEvent {
    Tick,
    Error,
    Render,
    User(KeyEvent),
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
// Styles
pub const INVALID_COLOR: Color = Color::LightRed;
pub const SELECTED_COLOR: Color = Color::LightBlue;
pub const PLACEHOLDER_COLOR: Color = Color::DarkGray;

// Keyboard input
pub const HELP_CHAR: char = 'h';
pub const QUIT_CHAR: char = 'q';
pub const MENU_CHAR: char = 'n';
pub const PAUSE_CHAR: char = 'p';
pub const RESUME_CHAR: char = 'r';
pub const DEVICE_LIST_CHAR: char = 'l';
pub const NEXT_ELEMENT_CHAR: char = ']';
pub const PREVIOUS_ELEMENT_CHAR: char = '[';

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

impl Parity {
    pub fn to_string(self) -> String {
        match self {
            Parity::Even => String::from("Even"),
            Parity::Odd => String::from("Odd"),
            Parity::None => String::from("None"),
        }
    }
}

impl Mode {
    pub fn to_string(self) -> String {
        match self {
            Mode::Ascii => String::from("Ascii"),
            Mode::Decimal => String::from("Decimal"),
            Mode::Hex => String::from("Hex"),
            Mode::Octal => String::from("Octal"),
        }
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

pub fn serial_buffer_default() -> SerialBuffer {
    let mutex = Mutex::new(Vec::new());
    return Arc::new(mutex);
}

pub fn serial_error_default() -> SerialError {
    let mutex = Mutex::new(None);
    return Arc::new(mutex);
}

pub fn serial_flag_default() -> SerialFlag {
    return Arc::new(Mutex::new(false));
}

pub fn serial_params_default() -> SerialParams {
    let parameters = PortParameters::default();
    let mutex = Mutex::new(parameters);
    return Arc::new(mutex);
}

/******************************************************************************/
/*******************************************************************************
* Tests
*******************************************************************************/
/******************************************************************************/
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn test_get_center_bounds() {
        let test_area = Rect::new(0, 0, 80, 24);
        let expected_bounds = Rect::new(20, 6, 40, 12);
        let actual_bounds = get_center_bounds(50, 50, test_area);
        assert_eq!(expected_bounds, actual_bounds);
    }

    #[test]
    fn test_mode_to_string() {
        let mut mode = Mode::Ascii;
        assert_eq!(mode.to_string(), "Ascii");
        mode = Mode::Decimal;
        assert_eq!(mode.to_string(), "Decimal");
        mode = Mode::Hex;
        assert_eq!(mode.to_string(), "Hex");
        mode = Mode::Octal;
        assert_eq!(mode.to_string(), "Octal");
    }

    #[test]
    fn test_port_parameters_name() {
        let mut parameters = PortParameters::default();
        assert_eq!(parameters.name, None);
        parameters = parameters.name(String::from("test"));
        assert_eq!(parameters.name, Some(String::from("test")));
    }
    
    #[test]
    fn test_parity_to_string() {
        let mut parity = Parity::None;
        assert_eq!(parity.to_string(), "None");
        parity = Parity::Even;
        assert_eq!(parity.to_string(), "Even");
        parity = Parity::Odd;
        assert_eq!(parity.to_string(), "Odd");
    }    
}
