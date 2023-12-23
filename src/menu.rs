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

        let input_gap = 8;
        let input_width = 14;
        let min_width = (input_width * 2) + input_gap;

        let mut split = true;
        let mut min_height = (self.inputs.len() / 2) * 3 + 2;

        if bounds.width < min_width {
            min_height = self.inputs.len() * 3 + 2;
            split = false;
        }

        let title = Block::default()
            .title("Menu".to_string())
            .title_alignment(Alignment::Center)
            .title_style(Style::default().add_modifier(Modifier::BOLD));

        frame.render_widget(title, layout[0]);

        let elements = input_elements(
            &mut self.inputs,
            self.selected.into(),
            split,
            input_width.into(),
            input_gap.into(),
        );
        let menu = Paragraph::new(elements)
            .scroll((0, 0))
            .alignment(Alignment::Center);

        frame.render_widget(menu, layout[2]);

        if usize::from(bounds.height) <= min_height {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .track_symbol(None)
                .thumb_symbol("");

            frame.render_stateful_widget(scrollbar, layout[2], &mut self.scroll);
        }
    }
}

fn input_elements(
    inputs: &mut Vec<MenuInput>,
    _selected: usize,
    split: bool,
    width: usize,
    gap: usize,
) -> Vec<Line> {
    let gap_fmt = " ".repeat(gap);
    let underline_fmt = "▔".repeat(width);

    let mut elements = Vec::new();

    let iterations = if split {
        inputs.len() / 2
    } else {
        inputs.len()
    };

    for i in 0..iterations {
        let title: Line;
        let input: Line;
        let underline: Line;

        if split {
            title = Line::from(format!(
                "{: <w$}{}{: <w$}",
                inputs[i].title.to_string(),
                &gap_fmt,
                inputs[i + 3].title.to_string(),
                w = width
            ));
            input = Line::from(format!(
                "{: <w$}{}{: <w$}",
                input_text(&mut inputs[i]),
                &gap_fmt,
                input_text(&mut inputs[i + 3]),
                w = width
            ));
            underline = Line::from(format!("{}{}{}", underline_fmt, gap_fmt, underline_fmt));
        } else {
            title = Line::from(inputs[i].title.to_string());
            input = Line::from(input_text(&mut inputs[i]));
            underline = Line::from(underline_fmt.clone());
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
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Min(3),
        ])
        .split(bounds);

    return (bounds, layout);
}
