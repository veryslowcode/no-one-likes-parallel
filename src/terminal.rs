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
    buffer: Vec<DataByte>,
    scroll: ScrollbarState,
    parameters: PortParameters,
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
            offset: 0,
            buffer: vec![
                DataByte {
                    value: 0xA5,
                    direction: DataDirection::Output
                };
                1024
            ],
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

        if self.state == State::Pausing {
            render_pause(frame, self.bounds);
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
fn get_encoding(model: &mut TerminalModel, mode: Mode) -> Vec<Span> {
    let style = Style::default().fg(crate::PLACEHOLDER_COLOR);
    //    let mode = model.parameters.mode.clone().unwrap();
    model
        .buffer
        .iter()
        .map(|data_byte| match mode {
            Mode::Hex => {
                let text = format!("{:#X} ", data_byte.value);
                match data_byte.direction {
                    DataDirection::Input => Span::styled(text, style),
                    DataDirection::Output => Span::from(text),
                }
            }
            Mode::Octal => {
                let text = format!("{:#o} ", data_byte.value);
                match data_byte.direction {
                    DataDirection::Input => Span::styled(text, style),
                    DataDirection::Output => Span::from(text),
                }
            }
            // TODO implement ascii
            _ => Span::from(data_byte.value.to_string() + " "),
        })
        .collect()
}

fn get_layout(fsize: Rect) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(fsize)
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
    let line = Line::from(get_encoding(model, Mode::Octal));
    let terminal = Paragraph::new(line).block(block).wrap(Wrap { trim: false });
    frame.render_widget(terminal, area);
}

fn update_buffer_input(model: &mut TerminalModel) {
    let input_bytes = model.input.clone().into_bytes();
    for value in input_bytes.iter() {
        model.buffer.push(DataByte {
            value: *value,
            direction: DataDirection::Input,
        });
    }
}

fn update_buffer_output() {}
