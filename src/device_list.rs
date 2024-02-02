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
    widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
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
#[derive(Clone, Debug, PartialEq)]
pub struct DeviceListModel {
    state: State,
    bounds: Rect,
    offset: usize,
    selected: usize,
    devices: Vec<String>,
    scroll: ScrollbarState,
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
* Local Constants
*******************************************************************************/
/******************************************************************************/
const CONTENT_LENGTH: usize = 20;
const MARGIN_TOP: usize = 2;

/******************************************************************************/
/*******************************************************************************
* Implementation
*******************************************************************************/
/******************************************************************************/
impl Default for DeviceListModel {
    fn default() -> DeviceListModel {
        DeviceListModel {
            offset: 0,
            selected: 0,
            devices: Vec::new(),
            state: State::Running,
            bounds: Rect::default(),
            scroll: ScrollbarState::default()
                .content_length(CONTENT_LENGTH),
        }
    }
}

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
                switch_screen(self);
            }
            _ => {}
        }
        return self.get_state();
    }

    fn view(&mut self, frame: &mut Frame) {
        let (bounds, layout) = get_layout(frame.size());
        self.bounds = bounds;

        render_title(frame, layout[0]);

        self.devices = get_available_devices().expect("Failed to determine available devices");

        render_device_list(frame, layout[2], self);
        render_scrollbar(frame, layout[2], self);
    }
}

/******************************************************************************/
/*******************************************************************************
* Utility Functions
*******************************************************************************/
/******************************************************************************/
fn get_layout(fsize: Rect) -> (Rect, Rc<[Rect]>) {
    let bounds = get_center_bounds(50, 50, fsize);
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(MARGIN_TOP as u16),
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

    if model.bounds.height <= CONTENT_LENGTH as u16 {
        model.offset = model.selected;
        model.scroll = model.scroll.position(model.offset);
    }
}

fn render_device_list(frame: &mut Frame, area: Rect, model: &mut DeviceListModel) {
    let mut text: Vec<Line> = Vec::new();

    if model.devices.len() > 0 {
        let style = Style::default().fg(crate::SELECTED_COLOR);
        for (index, name) in model.devices.iter().enumerate() {
            if index == model.selected {
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
        .scroll((model.offset as u16, 0))
        .alignment(Alignment::Center);

    frame.render_widget(list, area);
}

fn render_scrollbar(frame: &mut Frame, area: Rect, model: &mut DeviceListModel) {
    if usize::from(area.height) <= CONTENT_LENGTH && model.devices.len() > 0 {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .track_symbol(None)
            .thumb_symbol("");

        frame.render_stateful_widget(scrollbar, area, &mut model.scroll);
    }
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Block::default()
        .title("Device List")
        .title_alignment(Alignment::Center)
        .title_style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(title, area);
}

fn switch_screen(model: &mut DeviceListModel) {
    if model.devices.len() > 0 {
        let port_name = model.devices[model.selected].clone();
        model.state = State::Switching(
            Screen::Menu,
            Some(PortParameters::default().name(port_name)),
        );
    } else {
        model.state = State::Switching(Screen::Menu, None);
    }
}

/******************************************************************************/
/*******************************************************************************
* Tests
*******************************************************************************/
/******************************************************************************/
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_model() {
        let test_model = DeviceListModel::default();
        assert_eq!(test_model.get_state(), State::Running);
        assert_eq!(
            test_model.scroll, 
            ScrollbarState::default()
                .content_length(CONTENT_LENGTH)
        );
        assert_eq!(test_model.offset, 0);
        assert_eq!(test_model.selected, 0);
    }

    #[test]
    fn test_get_state() {
        let mut test_model = DeviceListModel::default();
        test_model.state = State::Running;
        assert_eq!(test_model.get_state(), State::Running);
    }

    #[test]
    fn test_set_state() {
        let mut test_model = DeviceListModel::default();
        test_model.set_state(State::Stopping);
        assert_eq!(test_model.state, State::Stopping);
    }

    #[test]
    fn test_update() {
        let mut test_model = DeviceListModel::default();

        test_model.update(Message::NextElement);
        assert_eq!(test_model.get_state(), State::Running);

        test_model.update(Message::PreviousElement);
        assert_eq!(test_model.get_state(), State::Running);
        
        test_model.update(Message::Pause);
        assert_eq!(test_model.get_state(), State::Running);
        
        test_model.update(Message::Quit);
        assert_eq!(test_model.get_state(), State::Running);
        
        test_model.update(Message::Rx(vec![0]));
        assert_eq!(test_model.get_state(), State::Running);
      
        test_model.update(Message::Enter);
        assert_eq!(
            test_model.get_state(), 
            State::Switching(Screen::Menu, None)
        );
    }

    #[test]
    fn test_select_element() {
        let mut test_model = DeviceListModel::default();
        test_model.bounds = Rect::new(0, 0, 80, (CONTENT_LENGTH - 1) as u16);
        
        select_element(&mut test_model, SelectElement::Next);
        assert_eq!(test_model.selected, 0);
        
        select_element(&mut test_model, SelectElement::Previous);
        assert_eq!(test_model.selected, 0);

        let mut test_devices = Vec::new();
        for i in 0..(CONTENT_LENGTH + 1) {
           test_devices.push(format!("test-device/{}", i));
        }
        test_model.devices = test_devices;

        select_element(&mut test_model, SelectElement::Next);
        assert_eq!(test_model.selected, 1);
        
        select_element(&mut test_model, SelectElement::Previous);
        assert_eq!(test_model.selected, 0);

        select_element(&mut test_model, SelectElement::Next);
        assert_eq!(test_model.offset, 1);

        test_model.selected = 0;
        select_element(&mut test_model, SelectElement::Previous);
        assert_eq!(test_model.selected, test_model.devices.len() - 1);
       
        test_model.selected = test_model.devices.len() - 1;
        select_element(&mut test_model, SelectElement::Next);
        assert_eq!(test_model.selected, 0);
    }
}
