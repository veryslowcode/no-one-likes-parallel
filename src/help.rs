/*******************************************************************************/
/********************************************************************************
* DESCRIPTION: Defines the help 'view', which provides a list of commands and
* information about the application.
* AUTHOR: jb
* DATE: 12/31/23
********************************************************************************/
/*******************************************************************************/
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use std::rc::Rc;

use crate::common::*;
/******************************************************************************/
/*******************************************************************************
* Public Interface
*******************************************************************************/
/******************************************************************************/
#[derive(Clone, Debug, PartialEq)]
pub struct HelpModel {
    state: State,
    bounds: Rect,
    offset: usize,
    caller: Screen,
    scroll: ScrollbarState,
    pub parameters: Option<PortParameters>,
}

/******************************************************************************/
/*******************************************************************************
* Local Constants
*******************************************************************************/
/******************************************************************************/
const CONTENT_LENGTH: usize = 18;
const MARGIN_TOP: usize = 2;

/******************************************************************************/
/*******************************************************************************
* Implementation
*******************************************************************************/
/******************************************************************************/
impl Default for HelpModel {
    fn default() -> HelpModel {
        HelpModel {
            offset: 0,
            parameters: None,
            caller: Screen::Menu,
            state: State::Running,
            bounds: Rect::default(),
            scroll: ScrollbarState::default().content_length(CONTENT_LENGTH),
        }
    }
}

impl HelpModel {
    pub fn new(caller: Screen, parameters: Option<PortParameters>) -> HelpModel {
        let mut model = HelpModel::default();
        model.parameters = parameters;
        model.caller = caller;
        return model;
    }
}

impl Nolp for HelpModel {
    fn get_state(&self) -> State {
        return self.state.clone();
    }

    fn set_state(&mut self, s: State) {
        self.state = s;
    }
}

impl Tea for HelpModel {
    fn update(&mut self, msg: Message) -> State {
        match msg {
            Message::PreviousElement => {
                if self.offset == 0 {
                    self.offset = CONTENT_LENGTH;
                } else {
                    self.offset -= 1;
                }
                self.scroll = self.scroll.position(self.offset);
            }
            Message::NextElement => {
                if self.offset >= CONTENT_LENGTH {
                    self.offset = 0;
                } else {
                    self.offset += 1;
                }
                self.scroll = self.scroll.position(self.offset);
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
        render_help(frame, layout[2], self);
        render_scrollbar(frame, layout[2], self);
    }
}

/******************************************************************************/
/*******************************************************************************
* Utility Functions
*******************************************************************************/
/******************************************************************************/
fn get_input_info<'a>(width: usize) -> Vec<Line<'a>> {
    let mut info: Vec<Line> = Vec::new();
    let style = Style::default().fg(crate::PLACEHOLDER_COLOR);

    info.push(Line::from("Menu Input - Expected Value"));
    info.push(Line::from(""));

    info.push(Line::from(vec![
        Span::from(format!("{: <9}", "Port")),
        Span::styled(format!("{: >w$}", "Port name/path", w = width), style),
    ]));

    info.push(Line::from(vec![
        Span::from(format!("{: <9}", "Baudrate")),
        Span::styled(format!("{: >w$}", "Serialport baudrate", w = width), style),
    ]));

    info.push(Line::from(vec![
        Span::from("Data bits"),
        Span::styled(format!("{: >w$}", "5 - 8", w = width), style),
    ]));

    info.push(Line::from(vec![
        Span::from("Stop bits"),
        Span::styled(format!("{: >w$}", "1|2", w = width), style),
    ]));

    info.push(Line::from(vec![
        Span::from(format!("{: <9}", "Parity")),
        Span::styled(format!("{: >w$}", "None|Even|Odd", w = width), style),
    ]));

    info.push(Line::from(vec![
        Span::from(format!("{: <7}", "Mode")),
        Span::styled(
            format!("{: >w$}", "Ascii|Decimal|Hex|Octal", w = width),
            style,
        ),
    ]));

    return info;
}

fn get_keymap<'a>(width: usize) -> Vec<Line<'a>> {
    let mut keymap: Vec<Line> = Vec::new();
    let style = Style::default().fg(crate::PLACEHOLDER_COLOR);

    keymap.push(Line::from("Keymap (Views)"));
    keymap.push(Line::from(""));

    keymap.push(Line::from(vec![
        Span::from(format!("ctrl+{}", MENU_CHAR)),
        Span::styled(format!("{: >w$}", "Displays menu", w = width), style),
    ]));

    keymap.push(Line::from(vec![
        Span::from(format!("ctrl+{}", DEVICE_LIST_CHAR)),
        Span::styled(format!("{: >w$}", "Displays device list", w = width), style),
    ]));

    keymap.push(Line::from(vec![
        Span::from(format!("ctrl+{}", QUIT_CHAR)),
        Span::styled(format!("{: >w$}", "Quits application", w = width), style),
    ]));

    keymap.push(Line::from(""));
    keymap.push(Line::from("Keymap (Movement)"));
    keymap.push(Line::from(""));

    keymap.push(Line::from(vec![
        Span::from(format!("ctrl+{}", PREVIOUS_ELEMENT_CHAR)),
        Span::styled(format!("{: >w$}", "Previous/scroll up", w = width), style),
    ]));

    keymap.push(Line::from(vec![
        Span::from(format!("ctrl+{}", NEXT_ELEMENT_CHAR)),
        Span::styled(format!("{: >w$}", "Next/scroll down", w = width), style),
    ]));

    keymap.push(Line::from(""));

    return keymap;
}

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

fn render_help(frame: &mut Frame, area: Rect, model: &mut HelpModel) {
    let mut text: Vec<Line> = Vec::new();
    let width = 24;

    text.append(&mut get_keymap(width));
    text.append(&mut get_input_info(width - 3));

    let help = Paragraph::new(text)
        .scroll((model.offset as u16, 0))
        .alignment(Alignment::Center);

    frame.render_widget(help, area);
}

fn render_scrollbar(frame: &mut Frame, area: Rect, model: &mut HelpModel) {
    if usize::from(area.height) <= CONTENT_LENGTH {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .track_symbol(None)
            .thumb_symbol("");

        frame.render_stateful_widget(scrollbar, area, &mut model.scroll);
    }
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Block::default()
        .title("Help")
        .title_alignment(Alignment::Center)
        .title_style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(title, area);
}

fn switch_screen(model: &mut HelpModel) {
    model.state = State::Switching(model.caller.to_owned(), model.parameters.to_owned());
}
