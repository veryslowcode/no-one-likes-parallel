/*******************************************************************************/
/********************************************************************************
* DESCRIPTION: Entry point for the NOLP serial terminal application.
* AUTHOR: jb
* DATE: 12/30/23
********************************************************************************/
/*******************************************************************************/
use anyhow::{anyhow, Result};
use crossterm::{
    cursor,
    event::{self, Event, EventStream, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{FutureExt, StreamExt};
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
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tokio::{
    self, select,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
    time::interval,
};

mod common;
mod device_list;
mod help;
mod menu;
mod serial;
mod terminal;

use crate::common::*;
use crate::device_list::DeviceListModel;
use crate::help::HelpModel;
use crate::menu::MenuModel;
use crate::terminal::TerminalModel;

type NolpBackend = CrosstermBackend<Stdout>;
type NolpTerminal = Terminal<NolpBackend>;

/******************************************************************************/
/*******************************************************************************
* Internal Interface
*******************************************************************************/
/******************************************************************************/
#[derive(Debug)]
struct Scene {
    screen: Screen,
    help: Option<HelpModel>,
    menu: Option<MenuModel>,
    terminal: Option<TerminalModel>,
    device_list: Option<DeviceListModel>,
}

#[derive(Debug)]
#[allow(unused)]
struct EventListener {
    tick_rate: Duration,
    frame_rate: Duration,
    task: Option<JoinHandle<()>>,
    sender: UnboundedSender<NolpEvent>,
    receiver: UnboundedReceiver<NolpEvent>,
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
            terminal: None,
            device_list: None,
            screen: Screen::default(),
            menu: Some(MenuModel::default()),
        }
    }
}

impl EventListener {
    fn new() -> Self {
        let tick_rate = Duration::from_millis(250);
        let frame_rate = Duration::from_secs_f64(1.0 / 60.0);
        let (tx, rx) = unbounded_channel();
        let tx_handle = tx.clone();
        EventListener::start(tx, tick_rate, frame_rate);
        EventListener {
            sender: tx_handle,
            receiver: rx,
            task: None,
            frame_rate,
            tick_rate,
        }
    }

    fn handle_error(tx: &UnboundedSender<NolpEvent>) {
        tx.send(NolpEvent::Error)
            .expect("Failed to notify tokio error");
    }

    fn handle_event(tx: &UnboundedSender<NolpEvent>, event: Event) {
        if let Event::Key(k) = event {
            if k.kind == KeyEventKind::Press {
                tx.send(NolpEvent::User(k))
                    .expect("Failed to send user event");
            }
        }
    }

    fn start(tx: UnboundedSender<NolpEvent>, tick_rate: Duration, frame_rate: Duration) {
        tokio::spawn(async move {
            let mut reader = EventStream::new();
            let mut ticker = interval(tick_rate);
            let mut renderer = interval(frame_rate);
            loop {
                let tick_delay = ticker.tick();
                let render_delay = renderer.tick();
                let next = reader.next().fuse();
                select! {
                        event = next => {
                match event {
                    Some(Ok(e)) => {
                    EventListener::handle_event(&tx, e)
                    },
                    Some(Err(_)) => {
                    EventListener::handle_error(&tx);
                    },
                    None => {},
                }
                        },
                        _ = tick_delay => {
                tx.send(NolpEvent::Tick).expect("Failed to send tick event");
                        }
                        _ = render_delay => {
                tx.send(NolpEvent::Render).expect("Failed to send render event");
                        }
                    }
            }
        });
    }

    async fn listen(&mut self) -> Result<NolpEvent> {
        self.receiver
            .recv()
            .await
            .ok_or(anyhow!("Failed to receive event"))
    }
}

#[tokio::main]
async fn nolp_main(rx: Arc<Mutex<Vec<u8>>>, tx: Arc<Mutex<Vec<u8>>>) {
    set_panic_hook();

    let mut state = State::default();
    let mut scene = Scene::default();
    let mut listener = EventListener::new();
    let mut terminal = init_terminal().expect("Failed to initialize terminal");
    while state != State::Stopping {
        let event = listener.listen().await.unwrap();
        match event {
            NolpEvent::User(k) => match get_message(&mut scene, k) {
                Some(m) => match m {
                    Message::Quit => state = State::Stopping,
                    Message::Switching(s, p) => switch_screen(&mut scene, s, p),
                    ms => update(&mut scene, &mut state, ms),
                },
                None => {}
            },
            NolpEvent::Tick => {
                if scene.screen == Screen::Terminal {
                    let mut rx_lock = rx.try_lock();
                    if let Ok(ref mut mutex) = rx_lock {
                        if (**mutex).len() > 0 {
                            let message = Message::Rx((**mutex).clone());
                            update(&mut scene, &mut state, message);
                            (**mutex).clear();
                        }
                        drop(rx_lock);
                    }
                    let mut scene_handle = &mut scene;
                    let mut buffer = scene_handle.as_mut().terminal.unwrap().get_output_buffer();
                    if buffer.len() > 0 {
                        let mut tx_lock = tx.try_lock();
                        if let Ok(ref mut mutex) = tx_lock {
                            (**mutex).append(&mut buffer);
                            drop(tx_lock);
                            //                            scene.terminal.unwrap().clear_output_buffer();
                        }
                    }
                }
            }
            NolpEvent::Render => render(&mut terminal, &mut scene),
            _ => {}
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

fn get_info<'a>(model: &mut impl Nolp) -> Paragraph<'a> {
    let mut style = Style::default().fg(crate::PLACEHOLDER_COLOR);
    let mut message = format!(" Help (ctrl+{}) | Quit (ctrl+{}) ", HELP_CHAR, QUIT_CHAR);

    if let State::Error(m) = model.get_state() {
        style = style.fg(crate::INVALID_COLOR);
        message = m;
    }

    let commands = Line::styled(message, style);
    let help = Paragraph::new(commands).alignment(Alignment::Center);
    return help;
}

fn get_message(scene: &mut Scene, key: KeyEvent) -> Option<Message> {
    if key.modifiers == event::KeyModifiers::CONTROL {
        match key.code {
            KeyCode::Char(QUIT_CHAR) => {
                return Some(Message::Quit);
            }
            KeyCode::Char(DEVICE_LIST_CHAR) => {
                return Some(Message::Switching(Screen::DeviceList, None));
            }
            KeyCode::Char(MENU_CHAR) => {
                let parameters = get_parameters(scene);
                return Some(Message::Switching(Screen::Menu, parameters));
            }
            KeyCode::Char(HELP_CHAR) => {
                let parameters = get_parameters(scene);
                return Some(Message::Switching(Screen::Help, parameters));
            }
            _ => {}
        }
    }

    return match key.code {
        KeyCode::Char(PREVIOUS_ELEMENT_CHAR) => Some(Message::PreviousElement),
        KeyCode::Char(NEXT_ELEMENT_CHAR) => Some(Message::NextElement),
        KeyCode::Char(RESUME_CHAR) => Some(Message::Resume),
        KeyCode::Char(PAUSE_CHAR) => Some(Message::Pause),
        KeyCode::Char(input) => Some(Message::Input(input)),
        KeyCode::Backspace => Some(Message::Backspace),
        KeyCode::Enter => Some(Message::Enter),
        _ => None,
    };
}

fn get_parameters(scene: &mut Scene) -> Option<PortParameters> {
    match scene.screen {
        Screen::Help => scene.help.as_mut().unwrap().parameters.clone(),
        Screen::Terminal => Some(scene.terminal.as_mut().unwrap().parameters.clone()),
        _ => None,
    }
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
            let info = get_info(model);

            frame.render_widget(frame_border, frame.size());
            model.view(frame);
            frame.render_widget(info, layout[1]);
        })
        .expect("Failed to render frame");
}

fn render(terminal: &mut NolpTerminal, scene: &mut Scene) {
    match scene.screen {
        Screen::Menu => {
            let model = scene.menu.as_mut().unwrap();
            render_screen(terminal, model);
        }
        Screen::DeviceList => {
            let model = scene.device_list.as_mut().unwrap();
            render_screen(terminal, model);
        }
        Screen::Help => {
            let model = scene.help.as_mut().unwrap();
            render_screen(terminal, model);
        }
        Screen::Terminal => {
            let model = scene.terminal.as_mut().unwrap();
            render_screen(terminal, model);
        }
    };
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
                    model = MenuModel::new(p);
                }
                None => {
                    model = MenuModel::default();
                }
            }
            scene.help = None;
            scene.terminal = None;
            scene.device_list = None;
            scene.menu = Some(model);
        }
        Screen::DeviceList => {
            scene.menu = None;
            scene.help = None;
            scene.terminal = None;
            scene.device_list = Some(DeviceListModel::default());
        }
        Screen::Help => {
            scene.menu = None;
            scene.terminal = None;
            scene.device_list = None;
            scene.help = Some(HelpModel::new(scene.screen.clone(), parameters));
        }
        Screen::Terminal => {
            scene.help = None;
            scene.menu = None;
            scene.device_list = None;
            scene.terminal = Some(TerminalModel::new(
                parameters.expect("Failed to provide port parameters"),
            ));
        }
    }

    scene.screen = new;
}

fn update(scene: &mut Scene, state: &mut State, msg: Message) {
    match scene.screen {
        Screen::Menu => {
            let model = scene.menu.as_mut().unwrap();
            *state = model.update(msg);
        }
        Screen::DeviceList => {
            let model = scene.device_list.as_mut().unwrap();
            *state = model.update(msg);
        }
        Screen::Help => {
            let model = scene.help.as_mut().unwrap();
            *state = model.update(msg);
        }
        Screen::Terminal => {
            let model = scene.terminal.as_mut().unwrap();
            *state = model.update(msg);
        }
    };

    if let State::Switching(s, p) = state {
        let screen = s.clone();
        let parameters = p.clone();
        switch_screen(scene, screen, parameters);
        *state = State::Running;
    }
}

/******************************************************************************/
/*******************************************************************************
* Entry Point
*******************************************************************************/
/******************************************************************************/
fn main() {
    let rx_buffer: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let tx_buffer: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));

    let serial_rx = Arc::clone(&rx_buffer);
    let serial_tx = Arc::clone(&tx_buffer);
    thread::spawn(move || {
        let mut iteration = 0;
        let mut input_buffer = Vec::new();
        while iteration < 120 {
            let mut rx_lock = serial_rx.try_lock();
            if let Ok(ref mut mutex) = rx_lock {
                if input_buffer.len() > 0 {
                    (**mutex).append(&mut input_buffer);
                }
                (**mutex).push(0x00);
                drop(rx_lock);
            }
            iteration += 1;
            let mut tx_lock = serial_tx.try_lock();
            if let Ok(ref mut mutex) = tx_lock {
                if (**mutex).len() > 0 {
                    input_buffer.append(&mut (**mutex));
                    (**mutex).clear();
                    drop(tx_lock);
                }
            }
            thread::sleep(Duration::from_millis(500));
        }
    });

    let nolp_rx = Arc::clone(&rx_buffer);
    let nolp_tx = Arc::clone(&tx_buffer);
    nolp_main(nolp_rx, nolp_tx);
}
