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
use ratatui::{backend::CrosstermBackend, terminal::Terminal};
use std::{
    io::{stdout, Stdout},
    panic,
    time::Duration,
};
use tokio;

mod common;
mod device_list;
mod menu;
mod serial;

use crate::common::*;
use crate::device_list::DeviceListModel;
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
            KeyCode::Char('c') => {
                return Some(Message::Quit);
            }
            KeyCode::Char('l') => {
                return Some(Message::Switching(Screen::DeviceList, None));
            }
            KeyCode::Char('n') => {
                return Some(Message::Switching(Screen::Menu, None));
            }
            _ => {}
        }
    }

    return match key.code {
        KeyCode::Char('[') => Some(Message::PreviousElement),
        KeyCode::Char(']') => Some(Message::NextElement),
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

fn render_screen(terminal: &mut NolpTerminal, model: &mut impl Tea) {
    terminal
        .draw(|frame| model.view(frame))
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
            scene.device_list = None;
            scene.menu = Some(model);
        }
        Screen::DeviceList => {
            scene.menu = None;
            scene.device_list = Some(DeviceListModel::default());
        }
    }

    scene.screen = new;
}
