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
    offset: usize,
    buffer: Vec<String>,
    scroll: ScrollbarState,
    parameters: PortParameters,
}

/******************************************************************************/
/*******************************************************************************
* Local Constants
*******************************************************************************/
/******************************************************************************/
const PADDING_TOP: u16 = 2;
const PADDING_LEFT: u16 = 2;
const PADDING_RIGHT: u16 = 2;
const PADDING_BOTTOM: u16 = 1;

/******************************************************************************/
/*******************************************************************************
* Implementation
*******************************************************************************/
/******************************************************************************/
impl Default for TerminalModel {
    fn default() -> TerminalModel {
        TerminalModel {
            offset: 0,
            buffer: Vec::new(),
            state: State::Running,
            bounds: Rect::default(),
            scroll: ScrollbarState::default(),
            parameters: PortParameters::default(),
        }
    }
}

impl TerminalModel {
    fn new(parameters: PortParameters) -> TerminalModel {
        // validate parameters
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
        return self.get_state();
    }

    fn view(&mut self, frame: &mut Frame) {
        // DEBUG CODE remove
        for i in 0..1024 {
            self.buffer.push(format!("String {}", i));
        }
        // ----------------

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
    let input = Paragraph::new("").block(block);
    frame.render_widget(input, area);
}

fn render_terminal(frame: &mut Frame, area: Rect, model: &mut TerminalModel) {
    let block = Block::default().padding(Padding::new(
        PADDING_LEFT,
        PADDING_RIGHT,
        PADDING_TOP,
        PADDING_BOTTOM,
    ));
    let terminal = Paragraph::new(model.buffer.join(""))
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(terminal, area);
}
