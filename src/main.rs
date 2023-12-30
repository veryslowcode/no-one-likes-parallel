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

use crate::common::*;
use crate::device_list::DeviceListModel;
use crate::menu::MenuModel;

type NolpBackend = CrosstermBackend<Stdout>;
type NolpTerminal = Terminal<NolpBackend>;

#[derive(Debug, PartialEq)]
struct Scene {
    state: State,
    menu: Option<MenuModel>,
    device_list: Option<DeviceListModel>,
}

impl Default for Scene {
    fn default() -> Scene {
        Scene {
            device_list: None,
            state: State::Running,
            menu: Some(MenuModel::default()),
        }
    }
}

#[tokio::main]
async fn main() {
    set_panic_hook();

    let mut scene = Scene::default();
    let mut terminal = init_terminal().expect("Failed to initialize terminal");

    while scene.state != State::Stopping {
        let msg = handle_event().expect("Failed to poll events");

        match msg {
            Some(m) => match m {
                Message::Switching(s) => scene = switch_screen(s),
                Message::Quit => scene.state = State::Stopping,
                m => render_scene(&mut terminal, scene, m),
            },
            None => {}
        }
    }

    reset_terminal().expect("Failed to reset terminal");
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
            KeyCode::Char('c') => {
                return Some(Message::Quit);
            }
            KeyCode::Char('p') => {
                return Some(Message::Switching(Screen::DeviceList));
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

fn render_scene(terminal: &mut NolpTerminal, scene: Scene, msg: Message) {
    if scene.menu.is_some() {
        let mut model = scene.menu.unwrap();
        terminal
            .draw(|frame| model.view(frame))
            .expect("Failed to render frame");

        model.update(msg)
    } else {
        let mut model = scene.device_list.unwrap();
        terminal
            .draw(|frame| model.view(frame))
            .expect("Failed to render frame");

        model.update(msg)
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

fn switch_screen(screen: Screen) -> Scene {
    let mut scene = Scene::default();
    match screen {
        Screen::Menu => {}
        Screen::DeviceList => {
            scene.menu = None;
            scene.device_list = Some(DeviceListModel::default());
        }
    }
    return scene;
}
