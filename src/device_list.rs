/*******************************************************************************/
/********************************************************************************
* DESCRIPTION: Defines the device-list 'view', which allows the user to see
* a list of available devices and/or choose one.
* AUTHOR: jb
* DATE: 12/30/23
********************************************************************************/
/*******************************************************************************/
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Paragraph, Scrollbar, ScrollbarState},
    Frame,
};
use std::rc::Rc;

use crate::common::*;
use crate::serial::get_available_devices;

/******************************************************************************/
/*******************************************************************************
* Public Interface
*******************************************************************************/
/******************************************************************************/
#[derive(Debug, PartialEq)]
pub struct DeviceListModel {
    state: State,
    bounds: Rect,
    offset: usize,
    selected: usize,
    devices: Vec<String>,
    scroll: ScrollbarState,
}

impl Default for DeviceListModel {
    fn default() -> DeviceListModel {
        DeviceListModel {
            offset: 0,
            selected: 0,
            devices: Vec::new(),
            state: State::Running,
            scroll: ScrollbarState::default().content_length(19),
            bounds: Rect::new(0, 0, 0, 0),
        }
    }
}

/******************************************************************************/
/*******************************************************************************
* Internal Interface
*******************************************************************************/
/******************************************************************************/
#[derive(Debug, PartialEq)]
enum SelectElement {
    Previous,
    Next,
}

/******************************************************************************/
/*******************************************************************************
* Implementation
*******************************************************************************/
/******************************************************************************/
impl Nolp for DeviceListModel {
    fn get_state(&self) -> State {
        return self.state.clone();
    }

    fn set_state(&mut self, s: State) {
        self.state = s;
    }
}

impl Tea for DeviceListModel {
    fn update(&mut self, msg: Message) -> State {
        match msg {
            Message::PreviousElement => {
                select_element(self, SelectElement::Previous);
            }
            Message::NextElement => {
                select_element(self, SelectElement::Next);
            }
            Message::Enter => {
                // TODO switch to menu with selected name
            }
            _ => {}
        }
        return self.get_state();
    }

    fn view(&mut self, frame: &mut Frame) {
        let (bounds, layout) = get_layout(frame.size(), 2);
        self.bounds = bounds;

        render_title(frame, layout[0]);

        self.devices = get_available_devices().expect("Failed to determine available devices");
        render_device_list(frame, layout[2], &self.devices, &self.selected);
    }
}

/******************************************************************************/
/*******************************************************************************
* Utility Functions
*******************************************************************************/
/******************************************************************************/
fn get_layout(fsize: Rect, margin_t: u16) -> (Rect, Rc<[Rect]>) {
    let bounds = get_center_bounds(50, 50, fsize);
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(margin_t),
            Constraint::Min(1),
        ])
        .split(bounds);
    return (bounds, layout);
}

fn select_element(model: &mut DeviceListModel, direction: SelectElement) {
    if model.devices.len() == 0 {
        return;
    }

    let (comparison, nominal, alternate) = match direction {
        SelectElement::Previous => (
            model.selected == 0,
            model.selected as i32 - 1,
            model.devices.len() - 1,
        ),
        SelectElement::Next => (
            model.selected == model.devices.len() - 1,
            model.selected as i32 + 1,
            0,
        ),
    };

    if comparison {
        model.selected = alternate;
    } else {
        model.selected = nominal as usize;
    }
}

fn render_device_list(frame: &mut Frame, area: Rect, devices: &Vec<String>, selected: &usize) {
    let mut text: Vec<Line> = Vec::new();

    if devices.len() > 0 {
        let style = Style::default().fg(crate::SELECTED_COLOR);
        for (index, name) in devices.iter().enumerate() {
            if index == *selected {
                text.push(Line::styled(name.to_string(), style));
            } else {
                text.push(Line::from(name.to_string()));
            }
        }
    } else {
        let style = Style::default().fg(crate::INVALID_COLOR);
        text.push(Line::styled("No devices available", style));
    }

    let list = Paragraph::new(text)
        .scroll((0, 0))
        .alignment(Alignment::Center);

    frame.render_widget(list, area);
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Block::default()
        .title("Device List")
        .title_alignment(Alignment::Center)
        .title_style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(title, area);
}
