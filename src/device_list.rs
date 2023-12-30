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
                if self.devices.len() == 0 {
                    return self.get_state();
                }

                if self.selected == 0 {
                    self.selected = self.devices.len() - 1;
                } else {
                    self.selected -= 1;
                }
            }
            Message::NextElement => {
                if self.devices.len() == 0 {
                    return self.get_state();
                }

                if self.selected == self.devices.len() - 1 {
                    self.selected = 0;
                } else {
                    self.selected += 1;
                }
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
        let title = Block::default()
            .title("Device List")
            .title_alignment(Alignment::Center)
            .title_style(Style::default().add_modifier(Modifier::BOLD));

        self.devices = get_available_devices().expect("Failed to determine available devices");
        let mut text: Vec<Line> = Vec::new();

        if self.devices.len() > 0 {
            let style = Style::default().fg(crate::SELECTED_COLOR);
            for (index, name) in self.devices.iter().enumerate() {
                if index == self.selected {
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

        frame.render_widget(title, layout[0]);
        frame.render_widget(list, layout[2]);
    }
}

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
