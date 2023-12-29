use ratatui::Frame;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
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
    invalid: bool,
    placeholder: String,
}

#[derive(Debug, PartialEq)]
pub struct MenuModel {
    state: State,
    split: bool,
    bounds: Rect,
    selected: u8,
    offset: usize,
    scroll: ScrollbarState,
    inputs: Vec<MenuInput>,
}

impl Default for MenuInput {
    fn default() -> MenuInput {
        MenuInput {
            limit: 100,
            invalid: false,
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
                .limit(7)
                .title(String::from("Mode"))
                .placeholder(String::from("Ascii")),
        );

        MenuModel {
            scroll: ScrollbarState::default().content_length(19),
            bounds: Rect::new(0, 0, 0, 0),
            state: State::Running,
            split: true,
            selected: 0,
            offset: 0,
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
                    self.selected = (self.inputs.len() + 1) as u8;
                } else {
                    self.selected -= 1;
                }
                let min_height = if self.split { 9 } else { 19 };
                if self.bounds.height <= min_height {
                    (self.scroll, self.offset) =
                        update_scroll(self.scroll, self.selected, self.split);
                }
            }
            Message::NextElement => {
                if usize::from(self.selected) >= self.inputs.len() + 1 {
                    self.selected = 0;
                } else {
                    self.selected += 1;
                }
                let min_height = if self.split { 9 } else { 19 };
                if self.bounds.height <= min_height {
                    (self.scroll, self.offset) =
                        update_scroll(self.scroll, self.selected, self.split);
                }
            }
            Message::Input(input) => {
                let index = usize::from(self.selected);
                let limit = self.inputs[index].limit.into();
                let valid = check_valid_number_input(&input, &index);
                if self.inputs[index].value.len() < limit && valid {
                    self.inputs[index].value.push(input);
                }
            }
            Message::Backspace => {
                let index = usize::from(self.selected);
                if self.inputs[index].value.len() > 0 {
                    self.inputs[index].value.pop();
                }
            }
            Message::Enter => {
                if usize::from(self.selected) == self.inputs.len() {
                    self.set_state(State::Stopping);
                } else if usize::from(self.selected) == self.inputs.len() + 1 {
                    if !check_valid_inputs(&mut self.inputs) {
                        self.set_state(State::Switching);
                        // TODO open serial connection
                    }
                }
            }
            Message::Quit => self.set_state(State::Stopping),
        }
    }

    fn view(&mut self, frame: &mut Frame) {
        self.split = true;
        let mut dimensions = InputDimensions {
            split: self.split,
            gap: 10_usize,
            width: 18_usize,
        };
        let margin_top = 2_u16;
        let (bounds, layout) = get_menu_layout(frame.size(), margin_top);
        self.bounds = layout[2];

        // Width * 2 to account for side-by-side inputs
        let min_width = (dimensions.width * 2) + dimensions.gap;
        // Input count / 2 to account for split
        // Multiplied by 3 to account for input line count
        // Add 1 to account for buttons
        let mut min_height = (self.inputs.len() / 2) * 3 + usize::from(margin_top) + 1;

        if usize::from(bounds.width) < min_width {
            min_height = self.inputs.len() * 3 + usize::from(margin_top) + 2;
            self.split = false;
            dimensions.split = self.split;
        }

        let title = Block::default()
            .title("Menu".to_string())
            .title_alignment(Alignment::Center)
            .title_style(Style::default().add_modifier(Modifier::BOLD));

        frame.render_widget(title, layout[0]);

        let mut elements = get_input_elements(&self.inputs, &dimensions, self.selected.into());

        let mut buttons =
            get_button_elements(&dimensions, self.inputs.len() - 1, self.selected.into());

        elements.append(&mut buttons);

        let menu = Paragraph::new(elements)
            .scroll((self.offset as u16, 0))
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

fn check_valid_number_input(input: &char, selected: &usize) -> bool {
    match selected {
        1 => match input.to_digit(10) {
            Some(_) => return true,
            None => return false,
        },
        2 => match input.to_digit(10) {
            Some(v) => return (5..=8).contains(&v),
            None => return false,
        },
        3 => match input.to_digit(10) {
            Some(v) => return (1..=3).contains(&v),
            None => return false,
        },
        _ => return true,
    }
}

fn check_valid_inputs(inputs: &mut Vec<MenuInput>) -> bool {
    let mut valid = true;

    for i in 0..(inputs.len() - 2) {
        if inputs[i].value.is_empty() {
            inputs[i].invalid = true;
            valid = false;
        } else {
            inputs[i].invalid = false;
        }
    }

    match inputs[4].value.to_lowercase().as_str() {
        "even" | "odd" | "none" => inputs[4].invalid = false,
        _ => {
            inputs[4].invalid = true;
            valid = false;
        }
    }

    match inputs[5].value.to_lowercase().as_str() {
        "ascii" | "decimal" | "hex" | "octal" => inputs[5].invalid = false,
        _ => {
            inputs[5].invalid = true;
            valid = false;
        }
    }

    return valid;
}

fn get_button_elements<'a>(
    dimensions: &InputDimensions,
    input_count: usize,
    selected: usize,
) -> Vec<Line<'a>> {
    let mut buttons = Vec::new();

    let mut cancel = Span::from("Cancel");
    let mut start = Span::from("Start");

    let selected_style = Style::default().fg(Color::LightBlue);

    if selected == input_count + 1 {
        cancel.patch_style(selected_style);
    } else if selected == input_count + 2 {
        start.patch_style(selected_style);
    }

    if dimensions.split {
        let gap_span = Span::from(format!("{}", " ".repeat(dimensions.gap)));
        buttons.push(Line::from(vec![cancel, gap_span, start]));
    } else {
        buttons.push(Line::from(cancel));
        buttons.push(Line::from(start));
    }

    return buttons;
}

fn get_input_elements<'a>(
    inputs: &'a Vec<MenuInput>,
    dimensions: &InputDimensions,
    selected: usize,
) -> Vec<Line<'a>> {
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
    let mut span = MenuSpans {
        title: vec![Span::from(format!(
            "{: <w$}",
            input.title.to_string(),
            w = width
        ))],
        input: vec![Span::styled(format!("{: <w$}", text, w = width), style)],
        underline: vec![Span::from(underline)],
    };

    if input.invalid {
        let invalid_style = Style::default().fg(Color::LightRed);
        span.title[0].patch_style(invalid_style);
        span.underline[0].patch_style(invalid_style);
    }

    return span;
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

fn update_scroll(scroll: ScrollbarState, selected: u8, split: bool) -> (ScrollbarState, usize) {
    if split {
        let adjustment = if selected % 2 != 0 {
            selected - 1
        } else {
            selected
        };
        let offset = usize::from((adjustment / 2) * 3);
        return (scroll.position(offset), offset);
    } else {
        let offset = if selected != 7 {
            usize::from(selected * 3)
        } else {
            usize::from(selected * 3 - 2)
        };
        return (scroll.position(offset), offset);
    }
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

    if input.invalid {
        let invalid_style = Style::default().fg(Color::LightRed);
        spans.title[1].patch_style(invalid_style);
        spans.underline[1].patch_style(invalid_style);
    }
}
