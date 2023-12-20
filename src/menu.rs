use ratatui::Frame;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{
        Borders, Block, Paragraph, 
        Scrollbar, ScrollbarOrientation, ScrollbarState
    }
};

use crate::common::*;

#[derive(Debug, PartialEq)]
struct MenuInput {
    title: String,
    value: String,
    placeholder: String
}

#[derive(Debug, PartialEq)]
pub struct MenuModel {
    state: State,
    selected: u8,
    scroll: ScrollbarState,
    inputs: Vec<MenuInput>
}

impl Default for MenuInput {
    fn default() -> MenuInput {
        MenuInput {
            title: String::from(""),
            value: String::from(""),
            placeholder: String::from("")
        }
    }
}

impl MenuInput {
    fn title(mut self, s: String) -> Self {
        self.title = s;
        return self;
    }

    fn value(mut self, s: String) -> Self {
        self.value = s;
        return self;
    }

    fn placeholder(mut self, s: String) -> Self {
        self.placeholder = s;
        return self;
    }
}

impl Default for MenuModel {
    fn default() -> MenuModel {
        let mut inputs = Vec::new();
        inputs.push(
            MenuInput::default()
                .title(String::from("Port"))
                .placeholder(String::from("COM4"))
        );
        
        inputs.push(
            MenuInput::default()
                .title(String::from("Baudrate"))
                .placeholder(String::from("9600"))
        );

        MenuModel {
            scroll: ScrollbarState::default()
                .content_length(12),
            state: State::Running,
            selected: 0,
            inputs
        }
    }
}

impl Nolp for MenuModel {
    fn get_state(&mut self) -> &State {
        return &self.state;
    }

    fn set_state(&mut self, s: State) {
        self.state = s;
    }
}

impl Tea for MenuModel {
    fn update(&mut self, msg: Message) {
        match msg {
            Message::Quit => self.set_state(State::Stopping),
        }
    }

    fn view(&mut self, frame: &mut Frame) {
        let bounds = get_center_bounds(50, 50, frame.size());
        let scrollbar = Scrollbar::default()
            .begin_symbol(Some("↑"))
            .track_symbol(Some("-"))
            .thumb_symbol("░")
            .end_symbol(Some("↓"))
            .orientation(ScrollbarOrientation::VerticalRight);

        frame.render_stateful_widget(
            scrollbar,
            bounds,
            &mut self.scroll
        );

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(0)
            ])
            .split(bounds);

        for (index, input) in self.inputs.iter().enumerate() {
            let text = if input.value.len() > 0 {
                &input.value
            } else {
                &input.placeholder
            };
    
            let b = Block::default()
                .title(input.title.to_string())
                .borders(Borders::BOTTOM);
            let w = Paragraph::new(text.to_string())
                .block(b);

            frame.render_widget(w, layout[index]);
        }
    }
}
