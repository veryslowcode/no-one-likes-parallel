use ratatui::Frame;
use ratatui::{
    layout::{
        Alignment, Constraint, 
        Direction, Layout
    },
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{
        Block, Paragraph, Scrollbar, 
        ScrollbarOrientation, ScrollbarState
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

        inputs.push(
            MenuInput::default()
                .title(String::from("Data bits"))
                .placeholder(String::from("8"))
        );

        inputs.push(
            MenuInput::default()
                .title(String::from("Stop bits"))
                .placeholder(String::from("1"))
        );

        inputs.push(
            MenuInput::default()
                .title(String::from("Parity"))
                .placeholder(String::from("Event"))
        );
        
        inputs.push(
            MenuInput::default()
                .title(String::from("Mode"))
                .placeholder(String::from("Ascii"))
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

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1)
            ])
            .split(bounds);

        let title = Block::default()
            .title("Menu".to_string())
            .title_alignment(Alignment::Center)
        .title_style(Style::default()
                     .add_modifier(Modifier::BOLD));

        frame.render_widget(title, layout[0]);
        
        let mut elements = Vec::new();
        for input in self.inputs.iter() {
            let style = Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::UNDERLINED);
            elements.push(Line::from(input.title.to_string()));
            elements.push(
                Line::styled(input.placeholder.to_string(), style));
        }

        let menu = Paragraph::new(elements)
            .scroll((0, 0));

        frame.render_widget(menu, layout[2]);

        frame.render_stateful_widget(
            scrollbar,
            layout[2],
            &mut self.scroll
        );
    }
}
