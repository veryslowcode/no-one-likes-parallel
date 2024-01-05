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
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Padding, Paragraph},
    Frame,
};
use std::rc::Rc;

use crate::common::*;
/******************************************************************************/
/*******************************************************************************
* Public Interface
*******************************************************************************/
/******************************************************************************/
#[derive(Debug)]
pub struct TerminalModel {
    state: State,
    bounds: Rect,
    input: String,
    out: Vec<u8>,
    buffer: Vec<DataByte>,
    pub parameters: PortParameters,
}

/******************************************************************************/
/*******************************************************************************
* Internal Interface
*******************************************************************************/
/******************************************************************************/
#[derive(Clone, Debug, PartialEq)]
enum DataDirection {
    Input,
    Output,
}

#[derive(Clone, Debug, PartialEq)]
struct DataByte {
    value: u8,
    direction: DataDirection,
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
            out: Vec::new(),
            buffer: Vec::new(),
            state: State::Running,
            input: String::from(""),
            bounds: Rect::default(),
            parameters: PortParameters::default(),
        }
    }
}

impl TerminalModel {
    pub fn new(parameters: PortParameters) -> TerminalModel {
        let mut model = TerminalModel::default();
        model.parameters = parameters;
        return model;
    }

    pub fn get_output_buffer(self) -> Vec<u8> {
        return self.out;
    }

    pub fn clear_output_buffer(&mut self) {
        self.out.clear();
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
                if self.state != State::Pausing && self.input.len() < 50 {
                    self.input.push(input);
                }
            }
            Message::Backspace => {
                if self.input.len() > 0 && self.state != State::Pausing {
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
                if self.input.len() > 0 {
                    update_buffer_input(self);
                    self.input = String::from("");
                }
            }
            Message::Rx(data) => {
                for d in data {
                    self.buffer.push(DataByte {
                        value: d,
                        direction: DataDirection::Output,
                    });
                }
            }
            _ => {}
        }
        return self.get_state();
    }

    fn view(&mut self, frame: &mut Frame) {
        self.bounds = frame.size();
        let layout = get_layout(self.bounds);

        if self.state == State::Pausing {
            render_pause(frame, self.bounds);
        } else if let State::Error(_) = self.state {
            render_error(frame, self.bounds, self);
        } else {
            render_terminal(frame, layout[0], self);
            render_input(frame, layout[1], self);
        }
    }
}

/******************************************************************************/
/*******************************************************************************
* Utility functions
*******************************************************************************/
/******************************************************************************/
fn get_encoding(model: &mut TerminalModel, area: Rect) -> Vec<Line> {
    let style = Style::default().fg(crate::PLACEHOLDER_COLOR);
    let mode = model.parameters.mode.clone().unwrap();
    let mut encoding: Vec<Line> = Vec::new();
    let mut current: Vec<Span> = Vec::new();
    for data_byte in model.buffer.iter() {
        let (width, text) = match mode {
            Mode::Hex => (3, format!("{:#02X} ", data_byte.value)),
            Mode::Octal => (6, format!("{:#05o} ", data_byte.value)),
            _ => (4, format!("{:#03}", data_byte.value.to_string())),
        };

        if usize::from(area.width) <= (current.len() * width) + width {
            encoding.push(Line::from(current));
            current = Vec::new();
        }

        match data_byte.direction {
            DataDirection::Output => current.push(Span::from(text)),
            DataDirection::Input => current.push(Span::styled(text, style)),
        }
    }

    if current.len() > 0 {
        encoding.push(Line::from(current));
    }

    return encoding;
}

fn get_layout(fsize: Rect) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(fsize)
}

fn render_error(frame: &mut Frame, area: Rect, model: &mut TerminalModel) {
    let bounds = get_center_bounds(50, 50, area);
    let style = Style::default()
        .fg(crate::INVALID_COLOR)
        .add_modifier(Modifier::BOLD);
    let message = format!(
        "There was an error connecting to {}",
        model.parameters.name.clone().unwrap()
    );
    let text = Text::styled(message, style);
    let pause = Paragraph::new(text).alignment(Alignment::Center);
    frame.render_widget(pause, bounds);
}

fn render_input(frame: &mut Frame, area: Rect, model: &mut TerminalModel) {
    let text = if model.input.len() > 0 {
        Text::styled(
            model.input.clone(),
            Style::default().fg(crate::SELECTED_COLOR),
        )
    } else {
        Text::styled("...", Style::default().fg(crate::PLACEHOLDER_COLOR))
    };
    let block = Block::default()
        .title(" Input ")
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    let input = Paragraph::new(text).block(block);
    frame.render_widget(input, area);
}

fn render_pause(frame: &mut Frame, area: Rect) {
    let bounds = get_center_bounds(50, 50, area);
    let style = Style::default()
        .fg(crate::PLACEHOLDER_COLOR)
        .add_modifier(Modifier::BOLD);
    let text = Text::styled("PAUSED", style);
    let pause = Paragraph::new(text).alignment(Alignment::Center);
    frame.render_widget(pause, bounds);
}

fn render_terminal(frame: &mut Frame, area: Rect, model: &mut TerminalModel) {
    let block = Block::default().padding(Padding::uniform(PADDING));
    let data = get_encoding(model, area);
    let terminal = Paragraph::new(data).block(block);
    frame.render_widget(terminal, area);
}

fn update_buffer_input(model: &mut TerminalModel) {
    let input_bytes = model.input.clone().into_bytes();
    model.out.append(&mut input_bytes);
    let mode = model.parameters.mode.as_ref().unwrap();
    let text_width = match mode {
        Mode::Hex => 3,
        Mode::Octal => 6,
        _ => 4,
    };
    let width = usize::from(model.bounds.width);
    let height = usize::from(model.bounds.height);

    let rel_height = height - 5;
    let rel_width = width / text_width;
    let mut rel_length = (model.buffer.len() + input_bytes.len() - 1) / rel_width;
    if width % text_width != 0 {
        rel_length += 1;
    }

    if rel_length > rel_height {
        model.buffer = Vec::new();
    }

    for value in input_bytes.iter() {
        model.buffer.push(DataByte {
            value: *value,
            direction: DataDirection::Input,
        });
    }
}

fn update_buffer_output() {}
