use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyEventKind, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode,
        EnterAlternateScreen, LeaveAlternateScreen
    }
};
use ratatui::prelude::*;
use std::{
    panic, time::Duration,
    io::{stdout, Stdout},
};

type NolpBackend = CrosstermBackend<Stdout>;
type NolpTerminal = Terminal<NolpBackend>;

#[derive(Debug, PartialEq)]
enum Message {
    Quit,
    None
}

fn main() {
    set_panic_hook();

    init_terminal()
        .expect("Failed to initialize terminal");

    loop {
        let msg = handle_event()
            .expect("Failed to poll events");
        if msg == Message::Quit {
            break;
        }
    }

    reset_terminal()
        .expect("Failed to reset terminal");
}

fn handle_event() -> Result<Message> {
    let poll_rate = Duration::from_millis(250);
    if event::poll(poll_rate)? {
        let event_read = event::read()?;
        return match event_read {
            Event::Key(k) => Ok(handle_key_event(k)),
            _ => Ok(Message::None)
        }
    }

    Ok(Message::None)
}

fn handle_key_event(key: event::KeyEvent) -> Message {
    if key.kind != KeyEventKind::Press {
        return Message::None;
    }

    return match key.code {
        KeyCode::Char('q') => Message::Quit,
        _ => Message::None
    }
}

fn init_terminal() -> Result<NolpTerminal> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let backend = NolpBackend::new(stdout());
    let terminal = NolpTerminal::new(backend)?;
    Ok(terminal)
}

fn reset_terminal() -> Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
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
