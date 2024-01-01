/*******************************************************************************/
/********************************************************************************
* DESCRIPTION: Entry point for the NOLP serial terminal application.
* AUTHOR: jb
* DATE: 12/30/23
********************************************************************************/
/*******************************************************************************/
use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    terminal::Terminal,
    text::Line,
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};
use std::{
    io::{stdout, Stdout},
    panic,
    rc::Rc,
    time::Duration,
};
use tokio;

mod common;
mod device_list;
mod help;
mod menu;
mod serial;

use crate::common::*;
use crate::device_list::DeviceListModel;
use crate::help::HelpModel;
use crate::menu::MenuModel;

type NolpBackend = CrosstermBackend<Stdout>;
type NolpTerminal = Terminal<NolpBackend>;

/******************************************************************************/
/*******************************************************************************
* Internal Interface
*******************************************************************************/
/******************************************************************************/
#[derive(Debug, PartialEq)]
struct Scene {
    screen: Screen,
    help: Option<HelpModel>,
    menu: Option<MenuModel>,
    device_list: Option<DeviceListModel>,
}

/******************************************************************************/
/*******************************************************************************
* Implementation
*******************************************************************************/
/******************************************************************************/
impl Default for Scene {
    fn default() -> Scene {
        Scene {
            help: None,
            device_list: None,
            screen: Screen::default(),
            menu: Some(MenuModel::default()),
        }
    }
}

#[tokio::main]
async fn main() {
    set_panic_hook();

    let mut state = State::default();
    let mut scene = Scene::default();
    let mut terminal = init_terminal().expect("Failed to initialize terminal");

    while state != State::Stopping {
        let msg = handle_event().expect("Failed to poll events");

        match msg {
            Some(m) => match m {
                Message::Quit => state = State::Stopping,
                Message::Switching(s, p) => switch_screen(&mut scene, s, p),
                ms => render_and_update(&mut terminal, &mut scene, &mut state, Some(ms)),
            },
            None => render_and_update(&mut terminal, &mut scene, &mut state, None),
        }
    }

    reset_terminal().expect("Failed to reset terminal");
}

/******************************************************************************/
/*******************************************************************************
* Utility Functions
*******************************************************************************/
/******************************************************************************/
fn get_frame_border<'a>() -> Block<'a> {
    Block::default()
        .title(" NOLP ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
}

fn get_layout(frame: &mut Frame) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(frame.size())
}

fn get_message<'a>(model: &mut impl Nolp) -> Paragraph<'a> {
    let mut style = Style::default().fg(crate::PLACEHOLDER_COLOR);
    let mut message = String::from(format!(
        " Help (ctrl+{}) | Quit (ctrl+{}) ",
        HELP_CHAR, QUIT_CHAR
    ));

    if let State::Error(m) = model.get_state() {
        style = style.fg(crate::INVALID_COLOR);
        message = m;
    }

    let commands = Line::styled(message, style);
    let help = Paragraph::new(commands).alignment(Alignment::Center);
    return help;
}

fn handle_event() -> Result<Option<Message>> {
    let poll_rate = Duration::from_millis(250);
    if event::poll(poll_rate)? {
        let event_read = event::read()?;
        return match event_read {
            Event::Key(k) => Ok(handle_key_event(k)),
            _ => Ok(None),
        };
    }

    Ok(None)
}

fn handle_key_event(key: event::KeyEvent) -> Option<Message> {
    if key.kind != KeyEventKind::Press {
        return None;
    }

    if key.modifiers == event::KeyModifiers::CONTROL {
        match key.code {
            KeyCode::Char(QUIT_CHAR) => {
                return Some(Message::Quit);
            }
            KeyCode::Char(DEVICE_LIST_CHAR) => {
                return Some(Message::Switching(Screen::DeviceList, None));
            }
            KeyCode::Char(MENU_CHAR) => {
                return Some(Message::Switching(Screen::Menu, None));
            }
            KeyCode::Char(HELP_CHAR) => {
                return Some(Message::Switching(Screen::Help, None));
            }
            _ => {}
        }
    }

    return match key.code {
        KeyCode::Char(PREVIOUS_ELEMENT_CHAR) => Some(Message::PreviousElement),
        KeyCode::Char(NEXT_ELEMENT_CHAR) => Some(Message::NextElement),
        KeyCode::Char(input) => Some(Message::Input(input)),
        KeyCode::Backspace => Some(Message::Backspace),
        KeyCode::Enter => Some(Message::Enter),
        _ => None,
    };
}

fn init_terminal() -> Result<NolpTerminal> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    execute!(stdout(), cursor::Hide)?;
    let backend = NolpBackend::new(stdout());
    let terminal = NolpTerminal::new(backend)?;
    Ok(terminal)
}

fn reset_terminal() -> Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    execute!(stdout(), cursor::Show)?;
    disable_raw_mode()?;
    Ok(())
}

fn render_screen(terminal: &mut NolpTerminal, model: &mut (impl Tea + Nolp)) {
    terminal
        .draw(|frame| {
            let layout = get_layout(frame);
            let frame_border = get_frame_border();
            let message = get_message(model);

            frame.render_widget(frame_border, frame.size());
            model.view(frame);
            frame.render_widget(message, layout[1]);
        })
        .expect("Failed to render frame");
}

fn render_and_update(
    terminal: &mut NolpTerminal,
    scene: &mut Scene,
    state: &mut State,
    msg: Option<Message>,
) {
    match scene.screen {
        Screen::Menu => {
            let model = scene.menu.as_mut().unwrap();
            render_screen(terminal, model);
            if msg.is_some() {
                *state = model.update(msg.unwrap());
            }
        }
        Screen::DeviceList => {
            let model = scene.device_list.as_mut().unwrap();
            render_screen(terminal, model);
            if msg.is_some() {
                *state = model.update(msg.unwrap());
            }
        }
        Screen::Help => {
            let model = scene.help.as_mut().unwrap();
            render_screen(terminal, model);
            if msg.is_some() {
                *state = model.update(msg.unwrap());
            }
        }
    };

    if let State::Switching(s, p) = state {
        let screen = s.clone();
        let parameters = p.clone();
        switch_screen(scene, screen, parameters);
    }
}

fn set_panic_hook() {
    let hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        reset_terminal().unwrap();
        eprintln!("An unexpected error occured");
        if cfg!(debug_assertions) {
            hook(info);
        } else {
            let err_msg = info.payload().downcast_ref::<&str>();
            match err_msg {
                Some(msg) => eprintln!("{:?}", msg),
                None => {}
            }
        }
    }));
}

fn switch_screen(scene: &mut Scene, new: Screen, parameters: Option<PortParameters>) {
    match new {
        Screen::Menu => {
            let model: MenuModel;
            match parameters {
                Some(p) => {
                    model = MenuModel::new(p.name.unwrap());
                }
                None => {
                    model = MenuModel::default();
                }
            }
            scene.help = None;
            scene.device_list = None;
            scene.menu = Some(model);
        }
        Screen::DeviceList => {
            scene.menu = None;
            scene.device_list = Some(DeviceListModel::default());
        }
        Screen::Help => {
            scene.menu = None;
            scene.device_list = None;
            scene.help = Some(HelpModel::new(scene.screen.clone()));
        }
    }

    scene.screen = new;
}
