/*******************************************************************************/
/********************************************************************************
* DESCRIPTION: Defines the menu 'view', which allows the user to specify serial
* port parameters.
* AUTHOR: jb
* DATE: 12/30/23
********************************************************************************/
/*******************************************************************************/
use ratatui::Frame;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};
use std::rc::Rc;

use crate::common::*;

/******************************************************************************/
/*******************************************************************************
* Public Interface
*******************************************************************************/
/******************************************************************************/
#[derive(Debug)]
struct MenuSpans<'a> {
    title: Vec<Span<'a>>,
    input: Vec<Span<'a>>,
    underline: Vec<Span<'a>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MenuInput {
    limit: usize,
    title: String,
    invalid: bool,
    pub value: String,
    placeholder: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MenuModel {
    state: State,
    split: bool,
    bounds: Rect,
    offset: usize,
    selected: usize,
    min_width: usize,
    min_height: usize,
    scroll: ScrollbarState,
    pub inputs: Vec<MenuInput>,
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

#[derive(Debug, PartialEq)]
enum UpdateElement {
    Sub,
    Add(char),
}

/******************************************************************************/
/*******************************************************************************
* Local Constants
*******************************************************************************/
/******************************************************************************/
const CONTENT_LENGTH: usize = 20;
const INPUT_WIDTH: usize = 18;
const MARGIN_TOP: usize = 2;
const GAP_WIDTH: usize = 10;

/******************************************************************************/
/*******************************************************************************
* Implementation
*******************************************************************************/
/******************************************************************************/
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

    fn limit(mut self, l: usize) -> Self {
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
            scroll: ScrollbarState::default().content_length(CONTENT_LENGTH),
            bounds: Rect::default(),
            state: State::Running,
            min_height: 0,
            min_width: 0,
            split: true,
            selected: 0,
            offset: 0,
            inputs,
        }
    }
}

impl MenuModel {
    pub fn new(parameters: PortParameters) -> MenuModel {
        let mut model = MenuModel::default();
        model.inputs[0].value = parameters.name.unwrap_or(String::from(""));
        model.inputs[1].value = match parameters.baud_rate {
            Some(b) => b.to_string(),
            None => String::from(""),
        };
        model.inputs[2].value = match parameters.data_bits {
            Some(d) => d.to_string(),
            None => String::from(""),
        };
        model.inputs[3].value = match parameters.stop_bits {
            Some(s) => s.to_string(),
            None => String::from(""),
        };
        model.inputs[4].value = match parameters.parity {
            Some(p) => p.to_string(),
            None => String::from(""),
        };
        model.inputs[5].value = match parameters.mode {
            Some(m) => m.to_string(),
            None => String::from(""),
        };
        return model;
    }
}

impl Nolp for MenuModel {
    fn get_state(&self) -> State {
        return self.state.clone();
    }

    fn set_state(&mut self, s: State) {
        self.state = s;
    }
}

impl Tea for MenuModel {
    fn update(&mut self, msg: Message) -> State {
        match msg {
            Message::PreviousElement => {
                select_element(self, SelectElement::Previous);
            }
            Message::NextElement => {
                select_element(self, SelectElement::Next);
            }
            Message::Input(input) => {
                update_element(self, UpdateElement::Add(input));
            }
            Message::Backspace => {
                update_element(self, UpdateElement::Sub);
            }
            Message::Enter => update_state(self),
            Message::Quit => self.set_state(State::Stopping),
            _ => {}
        }
        return self.get_state();
    }

    fn view(&mut self, frame: &mut Frame) {
        let (bounds, layout) = get_layout(frame.size());
        self.bounds = layout[2];

        update_split(self, bounds);

        render_title(frame, layout[0]);
        render_menu(frame, layout[2], self);
        render_scrollbar(frame, layout[2], self);
    }
}

/******************************************************************************/
/*******************************************************************************
* Utility Functions
*******************************************************************************/
/******************************************************************************/
fn get_button_elements<'a>(model: &mut MenuModel, selected_style: Style) -> Vec<Line<'a>> {
    let mut buttons = Vec::new();

    let mut cancel = Span::from("Cancel");
    let mut start = Span::from("Start");

    if model.selected == model.inputs.len() {
        cancel.patch_style(selected_style);
    } else if model.selected == model.inputs.len() + 1 {
        start.patch_style(selected_style);
    }

    if model.split {
        let gap_span = Span::from(format!("{}", " ".repeat(GAP_WIDTH)));
        buttons.push(Line::from(vec![cancel, gap_span, start]));
    } else {
        buttons.push(Line::from(cancel));
        buttons.push(Line::from(start));
    }

    return buttons;
}

fn get_input_elements<'a>(model: &'a mut MenuModel, selected_style: Style) -> Vec<Line<'a>> {
    let gap_fmt = " ".repeat(GAP_WIDTH);
    let underline_fmt = "▔".repeat(INPUT_WIDTH);

    let mut elements = Vec::new();

    let mut i = 0_usize;
    while i < model.inputs.len() {
        let increment: usize;
        let mut spans = get_input_spans(&model.inputs[i], underline_fmt.clone());

        match model.split {
            true => {
                update_spans_split(
                    &model.inputs[i + 1],
                    &mut spans,
                    &underline_fmt,
                    gap_fmt.clone(),
                );

                if i == model.selected {
                    spans.title[0].patch_style(selected_style);
                    spans.underline[0].patch_style(selected_style);
                } else if i + 1 == model.selected {
                    spans.title[1].patch_style(selected_style);
                    spans.underline[1].patch_style(selected_style);
                }

                increment = 2;
            }
            false => {
                if i == model.selected {
                    spans.title[0].patch_style(selected_style);
                    spans.underline[0].patch_style(selected_style);
                }

                increment = 1;
            }
        }

        elements.push(Line::from(spans.title));
        elements.push(Line::from(spans.input));
        elements.push(Line::from(spans.underline));

        i += increment;
    }

    return elements;
}

fn get_input_spans<'a>(input: &'a MenuInput, underline: String) -> MenuSpans<'a> {
    let (text, style) = get_input_text(&input);
    let mut span = MenuSpans {
        title: vec![Span::from(format!(
            "{: <w$}",
            input.title.to_string(),
            w = INPUT_WIDTH
        ))],
        input: vec![Span::styled(
            format!("{: <w$}", text, w = INPUT_WIDTH),
            style,
        )],
        underline: vec![Span::from(underline)],
    };

    if input.invalid {
        let invalid_style = Style::default().fg(crate::INVALID_COLOR);
        span.title[0].patch_style(invalid_style);
        span.underline[0].patch_style(invalid_style);
    }

    return span;
}

fn get_input_text(input: &MenuInput) -> (String, Style) {
    if input.value.len() > 0 {
        let text = &input.value.to_string();
        if input.value.len() >= INPUT_WIDTH {
            let slice_from = input.value.len() - INPUT_WIDTH;
            let slice = &text[slice_from..];
            return (slice.to_string(), Style::default());
        } else {
            return (text.to_string(), Style::default());
        }
    } else {
        return (
            input.placeholder.to_string(),
            Style::default().fg(crate::PLACEHOLDER_COLOR),
        );
    }
}

fn get_layout(fsize: Rect) -> (Rect, Rc<[Rect]>) {
    let bounds = get_center_bounds(50, 50, fsize);
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(MARGIN_TOP as u16),
            Constraint::Min(3),
        ])
        .split(bounds);

    return (bounds, layout);
}

fn get_port_parameters(model: &MenuModel) -> PortParameters {
    let baud_rate = model.inputs[1].value.parse::<u32>().unwrap();
    let data_bits = model.inputs[2].value.parse::<u8>().unwrap();
    let stop_bits = model.inputs[3].value.parse::<u8>().unwrap();
    let parity = match model.inputs[4].value.to_lowercase().as_str() {
        "even" => Parity::Even,
        "odd" => Parity::Odd,
        "none" => Parity::None,
        _ => unreachable!(),
    };
    let mode = match model.inputs[5].value.to_lowercase().as_str() {
        "ascii" => Mode::Ascii,
        "hex" => Mode::Hex,
        "decimal" => Mode::Decimal,
        "octal" => Mode::Octal,
        _ => unreachable!(),
    };

    return PortParameters {
        name: Some(model.inputs[0].value.clone()),
        baud_rate: Some(baud_rate),
        data_bits: Some(data_bits),
        stop_bits: Some(stop_bits),
        parity: Some(parity),
        mode: Some(mode),
    };
}

fn render_menu(frame: &mut Frame, area: Rect, model: &mut MenuModel) {
    let scroll_offset = model.offset;
    let mut model_handle = model.clone();
    let selected_style = Style::default().fg(crate::SELECTED_COLOR);
    let mut elements = get_input_elements(&mut model_handle, selected_style);
    let mut buttons = get_button_elements(model, selected_style);

    elements.append(&mut buttons);

    let menu = Paragraph::new(elements)
        .scroll((scroll_offset as u16, 0))
        .alignment(Alignment::Center);

    frame.render_widget(menu, area);
}

fn render_scrollbar(frame: &mut Frame, area: Rect, model: &mut MenuModel) {
    if usize::from(area.height) <= model.min_height {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .track_symbol(None)
            .thumb_symbol("");

        frame.render_stateful_widget(scrollbar, area, &mut model.scroll);
    }
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Block::default()
        .title("Menu".to_string())
        .title_alignment(Alignment::Center)
        .title_style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(title, area);
}

fn select_element(model: &mut MenuModel, direction: SelectElement) {
    // Note: the nominal case is cast to a signed integer to avoid potential
    // overflow, subtracting value from unsigned integer (e.g., 0 - 1)
    let (comparison, nominal, alternate) = match direction {
        SelectElement::Previous => (
            model.selected == 0,
            model.selected as i32 - 1,
            model.inputs.len() + 1,
        ),
        SelectElement::Next => (
            model.selected == model.inputs.len() + 1,
            model.selected as i32 + 1,
            0,
        ),
    };

    if comparison {
        model.selected = alternate;
    } else {
        model.selected = nominal as usize;
    }

    let min_height = if model.split {
        (CONTENT_LENGTH / 2) + 1
    } else {
        CONTENT_LENGTH
    };

    if model.bounds.height <= min_height as u16 {
        update_scroll(model);
    }
}

fn update_split(model: &mut MenuModel, area: Rect) {
    model.split = true;
    // Width * 2 to account for side-by-side inputs
    model.min_width = (INPUT_WIDTH * 2) + GAP_WIDTH;
    // Input count / 2 to account for split
    // Multiplied by 3 to account for input line count
    // Add 1 to account for buttons
    model.min_height = (model.inputs.len() / 2) * 3;

    if usize::from(area.width) < model.min_width {
        model.min_height = model.inputs.len() * 3 + 1;
        model.split = false;
    }
}

fn update_element(model: &mut MenuModel, update: UpdateElement) {
    match update {
        UpdateElement::Add(input) => {
            let is_valid = validate_input(model, input);
            let element = &model.inputs[model.selected];
            let within_limit = element.value.len() < element.limit;
            if within_limit && is_valid {
                model.inputs[model.selected].value.push(input);
            }
        }
        UpdateElement::Sub => {
            if model.inputs[model.selected].value.len() > 0 {
                model.inputs[model.selected].value.pop();
            }
        }
    }
}

fn update_scroll(model: &mut MenuModel) {
    match model.split {
        true => {
            let adjustment = if model.selected % 2 != 0 {
                model.selected - 1
            } else {
                model.selected
            };
            model.offset = (adjustment / 2) * 3;
            model.scroll = model.scroll.position(model.offset);
        }
        false => {
            model.offset = if model.selected != 7 {
                model.selected * 3
            } else {
                model.selected * 3 - 2
            };
            model.scroll = model.scroll.position(model.offset);
        }
    }
}

fn update_state(model: &mut MenuModel) {
    let cancel_btn = model.inputs.len();
    let start_btn = cancel_btn + 1;
    if model.selected == cancel_btn {
        model.set_state(State::Stopping);
    } else if model.selected == start_btn {
        if validate_values(model) {
            let parameters = get_port_parameters(model);
            model.set_state(State::Switching(Screen::Terminal, Some(parameters)));
        } else {
            model.set_state(State::Error(String::from(
                " Invalid input (ctrl+h) for help ",
            )));
        }
    }
}

fn update_spans_split(input: &MenuInput, spans: &mut MenuSpans, underline: &String, gap: String) {
    spans.title.push(Span::from(format!(
        "{}{: <w$}",
        &gap,
        input.title.to_string(),
        w = INPUT_WIDTH
    )));

    let (text, style) = get_input_text(&input);
    spans.input.push(Span::styled(
        format!("{}{: <w$}", &gap, text, w = INPUT_WIDTH),
        style,
    ));

    spans.underline.push(Span::from(format!(
        "{}{: <w$}",
        &gap,
        &underline,
        w = INPUT_WIDTH
    )));

    if input.invalid {
        let invalid_style = Style::default().fg(Color::LightRed);
        spans.title[1].patch_style(invalid_style);
        spans.underline[1].patch_style(invalid_style);
    }
}

fn validate_input(model: &mut MenuModel, input: char) -> bool {
    match model.selected {
        1 => match input.to_digit(10) {
            Some(_) => return true,
            None => return false,
        },
        2 => match input.to_digit(10) {
            Some(v) => return (5..=8).contains(&v),
            None => return false,
        },
        3 => match input.to_digit(10) {
            Some(v) => return (1..=2).contains(&v),
            None => return false,
        },
        _ => return true,
    }
}

fn validate_values(model: &mut MenuModel) -> bool {
    let mut valid = true;

    for i in 0..(model.inputs.len() - 2) {
        if model.inputs[i].value.is_empty() {
            model.inputs[i].invalid = true;
            valid = false;
        } else {
            model.inputs[i].invalid = false;
        }
    }

    match model.inputs[4].value.to_lowercase().as_str() {
        "even" | "odd" | "none" => model.inputs[4].invalid = false,
        _ => {
            model.inputs[4].invalid = true;
            valid = false;
        }
    }

    match model.inputs[5].value.to_lowercase().as_str() {
        "ascii" | "decimal" | "hex" | "octal" => model.inputs[5].invalid = false,
        _ => {
            model.inputs[5].invalid = true;
            valid = false;
        }
    }

    return valid;
}
