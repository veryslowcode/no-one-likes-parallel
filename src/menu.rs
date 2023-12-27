use ratatui::Frame;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};
use std::rc::Rc;

use crate::common::*;

#[derive(Debug)]
struct InputDimensions {
    gap: usize,
    width: usize,
    split: bool,
}

#[derive(Debug)]
struct MenuSpans<'a> {
    title: Vec<Span<'a>>,
    input: Vec<Span<'a>>,
    underline: Vec<Span<'a>>,
}

#[derive(Debug, PartialEq)]
struct MenuInput {
    limit: u8,
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
            limit: 100,
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

    fn placeholder(mut self, s: String) -> Self {
        self.placeholder = s;
        return self;
    }

    fn limit(mut self, l: u8) -> Self {
        self.limit = l;
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
                .limit(10)
                .title(String::from("Baudrate"))
                .placeholder(String::from("9600")),
        );

        inputs.push(
            MenuInput::default()
                .limit(1)
                .title(String::from("Data bits"))
                .placeholder(String::from("8")),
        );

        inputs.push(
            MenuInput::default()
                .limit(1)
                .title(String::from("Stop bits"))
                .placeholder(String::from("1")),
        );

        inputs.push(
            MenuInput::default()
                .limit(4)
                .title(String::from("Parity"))
                .placeholder(String::from("Even")),
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
            Message::PreviousElement => {
                if self.selected == 0 {
                    self.selected = (self.inputs.len() - 1) as u8;
                } else {
                    self.selected -= 1;
                }
            }
            Message::NextElement => {
                if usize::from(self.selected) >= self.inputs.len() - 1 {
                    self.selected = 0;
                } else {
                    self.selected += 1;
                }
            }
            Message::Input(input) => {
                let index = usize::from(self.selected);
                let limit = self.inputs[index].limit.into();
                if self.inputs[index].value.len() < limit {
                    self.inputs[index].value.push(input);
                }
            }
            Message::Backspace => {
                let index = usize::from(self.selected);
                if self.inputs[index].value.len() > 0 {
                    self.inputs[index].value.pop();
                }
            }
            Message::Quit => self.set_state(State::Stopping),
        }
    }

    fn view(&mut self, frame: &mut Frame) {
        let mut dimensions = InputDimensions {
            split: true,
            gap: 10_usize,
            width: 18_usize,
        };
        let margin_top = 2_u16;
        let (bounds, layout) = get_menu_layout(frame.size(), margin_top);

        // Width * 2 to account for side-by-side inputs
        let min_width = (dimensions.width * 2) + dimensions.gap;
        // Input count / 2 to account for split
        // Multiplied by 3 to account for input line count
        let mut min_height = (self.inputs.len() / 2) * 3 + usize::from(margin_top);

        if usize::from(bounds.width) < min_width {
            min_height = self.inputs.len() * 3 + usize::from(margin_top);
            dimensions.split = false;
        }

        let title = Block::default()
            .title("Menu".to_string())
            .title_alignment(Alignment::Center)
            .title_style(Style::default().add_modifier(Modifier::BOLD));

        frame.render_widget(title, layout[0]);

        let mut elements = get_input_elements(&self.inputs, dimensions, self.selected.into());

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

fn get_button_elements(
    dimensions: InputDimensions,
    element_count: usize,
    selected: usize,
) -> Vec<Line> {
    let mut buttons = Vec::new();

    let mut cancel = Span::from("Cancel");
    let mut start = Span::from("Start");

    let selected_style = Style::default().fg(Color::LightBlue);

    if selected == element_count {
        cancel.patch_style(selected_style);
    } else if selected == element_count + 1 {
        start.patch_style(selected_style);
    }

    if dimensions.split {
        let gap_span = Span::from(format!("{}", dimensions.gap.clone()));
        buttons.push(Line::from(vec![cancel, gap_span, start]));
    } else {
        buttons.push(Line::from(cancel));
        buttons.push(Line::from(start));
    }

    return buttons;
}

fn get_input_elements(
    inputs: &Vec<MenuInput>,
    dimensions: InputDimensions,
    selected: usize,
) -> Vec<Line> {
    let gap_fmt = " ".repeat(dimensions.gap);
    let underline_fmt = "▔".repeat(dimensions.width);

    let mut elements = Vec::new();

    let mut i = 0_usize;
    while i < inputs.len() {
        let mut spans = get_input_spans(&inputs[i], dimensions.width, underline_fmt.clone());

        if dimensions.split {
            update_spans_split(
                &inputs[i + 1],
                &mut spans,
                dimensions.width,
                underline_fmt.clone(),
                gap_fmt.clone(),
            );
        }

        let selected_style = Style::default().fg(Color::LightBlue);
        if usize::from(i) == selected {
            spans.title[0].patch_style(selected_style);
            spans.underline[0].patch_style(selected_style);
        } else if dimensions.split && usize::from(i + 1) == selected {
            spans.title[1].patch_style(selected_style);
            spans.underline[1].patch_style(selected_style);
        }

        elements.push(Line::from(spans.title));
        elements.push(Line::from(spans.input));
        elements.push(Line::from(spans.underline));

        if dimensions.split {
            i += 2;
        } else {
            i += 1;
        }
    }

    return elements;
}

fn get_input_spans<'a>(input: &'a MenuInput, width: usize, underline: String) -> MenuSpans<'a> {
    let (text, style) = get_input_text(&input, &width);
    MenuSpans {
        title: vec![Span::from(format!(
            "{: <w$}",
            input.title.to_string(),
            w = width
        ))],
        input: vec![Span::styled(format!("{: <w$}", text, w = width), style)],
        underline: vec![Span::from(underline)],
    }
}

fn get_input_text(input: &MenuInput, width: &usize) -> (String, Style) {
    if input.value.len() > 0 {
        let text = &input.value.to_string();
        let style = Style::default().fg(Color::White);
        if input.value.len() >= *width {
            let slice_from = input.value.len() - width;
            let slice = &text[slice_from..];
            return (slice.to_string(), style);
        } else {
            return (text.to_string(), style);
        }
    } else {
        return (
            input.placeholder.to_string(),
            Style::default().fg(Color::DarkGray),
        );
    }
}

fn get_menu_layout(fsize: Rect, margin_t: u16) -> (Rect, Rc<[Rect]>) {
    let bounds = get_center_bounds(50, 50, fsize);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(margin_t),
            Constraint::Min(3),
        ])
        .split(bounds);

    return (bounds, layout);
}

fn update_spans_split(
    input: &MenuInput,
    spans: &mut MenuSpans,
    width: usize,
    underline: String,
    gap: String,
) {
    spans.title.push(Span::from(format!(
        "{}{: <w$}",
        &gap,
        input.title.to_string(),
        w = width
    )));

    let (text, style) = get_input_text(&input, &width);
    spans.input.push(Span::styled(
        format!("{}{: <w$}", &gap, text, w = width),
        style,
    ));

    spans.underline.push(Span::from(format!(
        "{}{: <w$}",
        &gap,
        &underline,
        w = width
    )));
}
