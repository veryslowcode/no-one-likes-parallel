use ratatui::widgets::{Block, Paragraph, Scrollbar, ScrollbarState};
use ratatui::Frame;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
};
use std::rc::Rc;

use crate::common::*;

#[derive(Debug, PartialEq)]
pub struct DeviceListModel {
    state: State,
    bounds: Rect,
    offset: usize,
    devices: Vec<String>,
    scroll: ScrollbarState,
}

impl Default for DeviceListModel {
    fn default() -> DeviceListModel {
        DeviceListModel {
            offset: 0,
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
        return self.get_state();
    }

    fn view(&mut self, frame: &mut Frame) {
        let (bounds, layout) = get_layout(frame.size(), 2);
        self.bounds = bounds;
        let title = Block::default()
            .title("Device List")
            .title_alignment(Alignment::Center)
            .title_style(Style::default().add_modifier(Modifier::BOLD));

        let list = Paragraph::new("No devices available")
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
