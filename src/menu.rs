use ratatui::Frame;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};
use std::rc::Rc;

use crate::common::*;

#[derive(Debug, PartialEq)]
struct MenuInput {
    title: String,
    value: String,
    placeholder: String,
}

#[derive(Debug, PartialEq)]
pub struct MenuModel {
    state: State,
    selected: u8,
    scroll: ScrollbarState,
    inputs: Vec<MenuInput>,
}

impl Default for MenuInput {
    fn default() -> MenuInput {
        MenuInput {
            title: String::from(""),
            value: String::from(""),
            placeholder: String::from(""),
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
                .placeholder(String::from("COM4")),
        );

        inputs.push(
            MenuInput::default()
                .title(String::from("Baudrate"))
                .placeholder(String::from("9600")),
        );

        inputs.push(
            MenuInput::default()
                .title(String::from("Data bits"))
                .placeholder(String::from("8")),
        );

        inputs.push(
            MenuInput::default()
                .title(String::from("Stop bits"))
                .placeholder(String::from("1")),
        );

        inputs.push(
            MenuInput::default()
                .title(String::from("Parity"))
                .placeholder(String::from("Event")),
        );

        inputs.push(
            MenuInput::default()
                .title(String::from("Mode"))
                .placeholder(String::from("Ascii")),
        );

        MenuModel {
            scroll: ScrollbarState::default().content_length(12),
            state: State::Running,
            selected: 0,
            inputs,
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
        let (bounds, layout) = menu_layout(frame.size());

        let top_margin = 4;
        let input_width = 14;
        let input_width_gap = 4;
        let input_line_count = 3;

        let mut split = true;
        let mut min_height = ((self.inputs.len() * input_line_count) + top_margin) / 2;
        let min_width = (input_width * 2) + input_width_gap;
        if bounds.width < min_width {
            min_height = min_height * 2;
            split = false;
        }

        let title = Block::default()
            .title("Menu".to_string())
            .title_alignment(Alignment::Center)
            .title_style(Style::default().add_modifier(Modifier::BOLD));

        frame.render_widget(title, layout[0]);

        let elements = input_elements(&mut self.inputs, self.selected.into(), split);
        let menu = Paragraph::new(elements).scroll((0, 0));

        frame.render_widget(menu, layout[2]);

        if usize::from(bounds.height) < min_height {
            let scrollbar = Scrollbar::default()
                .begin_symbol(Some("↑"))
                .track_symbol(Some("-"))
                .thumb_symbol("░")
                .end_symbol(Some("↓"))
                .orientation(ScrollbarOrientation::VerticalRight);

            frame.render_stateful_widget(scrollbar, layout[2], &mut self.scroll);
        }
    }
}

fn input_elements(inputs: &mut Vec<MenuInput>, _selected: usize, split: bool) -> Vec<Line> {
    let mut elements = Vec::new();
    // let mut underline = Line::from(format!("{:▔>14}", ""));
    // let selected_style = Style::default().fg(Color::LightBlue);

    let iterations = if split {
        inputs.len() / 2
    } else {
        inputs.len()
    };

    for i in 0..iterations {
        let title: Line;
        let input: Line;
        let underline: Line;

        let underline_format = format!("{:▔>14}", "");

        if split {
            title = Line::from(format!(
                "{: <14}{: <8}{: <14}",
                inputs[i].title.to_string(),
                "",
                inputs[i + 3].title.to_string()
            ));
            input = Line::from(format!(
                "{: <14}{: <8}{: <14}",
                input_text(&mut inputs[i]),
                "",
                input_text(&mut inputs[i + 3])
            ));
            underline = Line::from(format!("{}{: <8}{}", underline_format, "", underline_format));
        } else {
            title = Line::from(inputs[i].title.to_string());
            input = Line::from(format!("{}", input_text(&mut inputs[i])));
            underline = Line::from(underline_format);
        }

        elements.push(title);
        elements.push(input);
        elements.push(underline);
    }

    return elements;
}

fn input_text(input: &mut MenuInput) -> String {
    if input.value.len() > 0 {
        return input.value.to_string();
    } else {
        return input.placeholder.to_string();
    }
}

fn menu_layout(fsize: Rect) -> (Rect, Rc<[Rect]>) {
    let bounds = get_center_bounds(50, 50, fsize);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(bounds);

    return (bounds, layout);
}
