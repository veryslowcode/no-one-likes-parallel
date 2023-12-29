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
mod menu;

use crate::common::*;
use crate::menu::MenuModel;

type NolpBackend = CrosstermBackend<Stdout>;
type NolpTerminal = Terminal<NolpBackend>;

#[tokio::main]
async fn main() {
    set_panic_hook();

    let mut model = MenuModel::default();
    let mut terminal = init_terminal().expect("Failed to initialize terminal");

    while model.get_state() != &State::Stopping {
        let msg = handle_event().expect("Failed to poll events");

        terminal
            .draw(|frame| model.view(frame))
            .expect("Failed to render frame");

        if msg.is_some() {
            model.update(msg.unwrap());
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
        if key.code == KeyCode::Char('c') {
            return Some(Message::Quit);
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
