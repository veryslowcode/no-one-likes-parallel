/*******************************************************************************/
/********************************************************************************
* DESCRIPTION: Defines the serial terminal 'view', which allows the user to
* interact with the open serial port.
* AUTHOR: jb
* DATE: 01/01/24
********************************************************************************/
/*******************************************************************************/
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{
        Block, BorderType, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Wrap,
    },
    Frame,
};
use std::rc::Rc;

use crate::common::*;
/******************************************************************************/
/*******************************************************************************
* Public Interface
*******************************************************************************/
/******************************************************************************/
#[derive(Debug, PartialEq)]
pub struct TerminalModel {
    state: State,
    bounds: Rect,
    input: String,
    offset: usize,
    buffer: Vec<u8>,
    scroll: ScrollbarState,
    parameters: PortParameters,
}

/******************************************************************************/
/*******************************************************************************
* Local Constants
*******************************************************************************/
/******************************************************************************/
const PADDING: u16 = 1;

/******************************************************************************/
/*******************************************************************************
* Implementation
*******************************************************************************/
/******************************************************************************/
impl Default for TerminalModel {
    fn default() -> TerminalModel {
        TerminalModel {
            offset: 0,
            buffer: vec![0xA5; 1024],
            state: State::Running,
            input: String::from(""),
            bounds: Rect::default(),
            scroll: ScrollbarState::default(),
            parameters: PortParameters::default(),
        }
    }
}

impl TerminalModel {
    fn new(parameters: PortParameters) -> TerminalModel {
        let mut model = TerminalModel::default();
        model.parameters = parameters;
        return model;
    }
}

impl Nolp for TerminalModel {
    fn get_state(&self) -> State {
        return self.state.clone();
    }

    fn set_state(&mut self, s: State) {
        self.state = s;
    }
}

impl Tea for TerminalModel {
    fn update(&mut self, msg: Message) -> State {
        match msg {
            Message::Input(input) => {
                self.input.push(input);
            }
            Message::Backspace => {
                if self.input.len() > 0 {
                    self.input.pop();
                }
            }
	    Message::Pause => {
		if self.state != State::Pausing {
		    // Clear the buffer
		    self.buffer = Vec::new();
		    self.state = State::Pausing;
		}
	    }
	    Message::Resume => {
		if self.state != State::Running {
		    self.state = State::Running;
		}
	    }
            Message::Enter => {
                // TODO send to port
		update_buffer_input(self);
                self.input = String::from("");
            }
            _ => {}
        }
        return self.get_state();
    }

    fn view(&mut self, frame: &mut Frame) {
        self.bounds = frame.size();
        let layout = get_layout(self.bounds);

        render_terminal(frame, layout[0], self);
        render_input(frame, layout[1], self);
    }
}

/******************************************************************************/
/*******************************************************************************
* Utility functions
*******************************************************************************/
/******************************************************************************/
fn get_layout(fsize: Rect) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(fsize)
}

fn render_input(frame: &mut Frame, area: Rect, model: &mut TerminalModel) {
    let block = Block::default()
        .title(" Input ")
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    let input = Paragraph::new(model.input.clone()).block(block);
    frame.render_widget(input, area);
}

fn render_terminal(frame: &mut Frame, area: Rect, model: &mut TerminalModel) {
    let block = Block::default().padding(Padding::uniform(PADDING));
    //    let mode = model.parameters.mode.clone().unwrap();
    let mode = Mode::Octal; // DEBUG CODE remove!
    let encoded: String = model
        .buffer
        .iter()
        .map(|value| match mode {
            Mode::Hex => format!("{:#X} ", value),
            Mode::Octal => format!("{:#o} ", value),
            // TODO implement ascii
            _ => value.to_string() + " ",
        })
        .collect();
    let terminal = Paragraph::new(encoded)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(terminal, area);
}

fn update_buffer_input(model: &mut TerminalModel) {
    let mut input_bytes = model.input.clone().into_bytes();
    model.buffer.append(&mut input_bytes);
}

fn update_buffer_output() {}
