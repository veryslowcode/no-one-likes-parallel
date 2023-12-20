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

#[derive(Debug, Default, PartialEq)]
enum ModelState {
   #[default]
    Running,
    Stopping 
}

#[derive(Debug, Default)]
struct Model {
    state: ModelState
}

#[derive(Debug, PartialEq)]
enum Message {
    Quit
}

fn main() {
    set_panic_hook();

    let mut model = Model::default();
    let _terminal = init_terminal()
        .expect("Failed to initialize terminal");

    while model.state != ModelState::Stopping {
        let msg = handle_event()
            .expect("Failed to poll events");

        // Call view
        
        if msg.is_some() {
            update(&mut model, msg.unwrap());
        }
    }

    reset_terminal()
        .expect("Failed to reset terminal");
}

fn handle_event() -> Result<Option<Message>> {
    let poll_rate = Duration::from_millis(250);
    if event::poll(poll_rate)? {
        let event_read = event::read()?;
        return match event_read {
            Event::Key(k) => Ok(handle_key_event(k)),
            _ => Ok(None)
        }
    }

    Ok(None)
}

fn handle_key_event(key: event::KeyEvent) -> Option<Message> {
    if key.kind != KeyEventKind::Press {
        return None;
    }

    return match key.code {
        KeyCode::Char('c') => {
            if key.modifiers == event::KeyModifiers::CONTROL {
                return Some(Message::Quit);
            } else {
                return None;
            }
        },
        _ => None
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

fn update(model: &mut Model, msg: Message) {
    match msg {
        Message::Quit => model.state = ModelState::Stopping,
    }
}
