use crate::emulator::EmulatorState;
use crossterm::event::{self, Event, KeyCode};
use emulator_state::EmulatorStateView;
use log::*;
use logging::LoggingView;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Tabs, Widget};
use ratatui::Frame;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::RwLock;
use std::thread::JoinHandle;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
    thread,
    time::Duration,
};
use tui_logger::{init_logger, set_default_level};

mod emulator_state;
mod logging;

const SNAPSHOT_DELAY_MS: u64 = 200;
const TUI_EVENT_POLL_MS: u64 = 4;

pub(super) trait Page {
    fn draw(&mut self, frame: &mut Frame, area: Rect);
    fn handle_event(&mut self, event: AppEvent);
}

enum Tab {
    EmulatorState(EmulatorStateView),
    Logging(LoggingView),
}

impl Tab {
    fn names() -> Vec<&'static str> {
        vec!["Emulator State", "Logging"]
    }

    fn as_index(&self) -> usize {
        match self {
            Self::EmulatorState(_) => 0,
            Self::Logging(_) => 1,
        }
    }

    fn handle_event(&mut self, event: AppEvent) {
        match self {
            Self::EmulatorState(emulator_state_page) => emulator_state_page.handle_event(event),
            Self::Logging(logging_state_page) => logging_state_page.handle_event(event),
        }
    }

    fn next_tab(&mut self, emulator_state_view: EmulatorStateView, logging_view: LoggingView) {
        *self = match self {
            Self::EmulatorState(_) => Self::Logging(logging_view),
            Self::Logging(_) => Self::EmulatorState(emulator_state_view),
        }
    }
}

pub(super) enum AppEvent {
    UiEvent(Event),
    StateEvent(EmulatorState),
}

pub struct Debugger {
    input_thread: JoinHandle<()>,
    snapshot_thread: JoinHandle<()>,
    tui_thread: JoinHandle<()>,
}

impl Debugger {
    pub fn new(
        emulator: &Arc<RwLock<EmulatorState>>,
        terminated: &Arc<AtomicBool>,
        paused: &Arc<AtomicBool>,
    ) -> Self {
        let (sender, receiver) = mpsc::channel();

        let input_thread = {
            let terminated_clone = terminated.clone();
            let event_sender = sender.clone();

            thread::spawn(move || run_input_thread(terminated_clone, event_sender))
        };

        let snapshot_thread = {
            let snapshot_sender = sender.clone();
            let terminated_clone = terminated.clone();
            let emulator_clone = emulator.clone();

            thread::spawn(move || {
                run_snapshot_thread(terminated_clone, emulator_clone, snapshot_sender)
            })
        };

        let tui_thread = {
            let emulator_state_view = EmulatorStateView::new(emulator.clone());
            let logging_view = LoggingView::new();
            let mut tab = Tab::EmulatorState(emulator_state_view.clone());
            let terminated_clone = terminated.clone();
            let paused_clone = paused.clone();

            thread::spawn(move || {
                run_tui_thread(
                    receiver,
                    terminated_clone,
                    paused_clone,
                    &mut tab,
                    emulator_state_view,
                    logging_view,
                )
            })
        };

        Self {
            input_thread,
            snapshot_thread,
            tui_thread,
        }
    }

    pub fn shutdown(self) {
        self.input_thread.join().unwrap();
        self.snapshot_thread.join().unwrap();
        self.tui_thread.join().unwrap();
    }
}

fn run_tui_thread(
    receiver: Receiver<AppEvent>,
    terminated: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    tab: &mut Tab,
    emulator_state_view: EmulatorStateView,
    logging_view: LoggingView,
) {
    init_logger(LevelFilter::Info).unwrap();
    set_default_level(LevelFilter::Info);

    let mut terminal = ratatui::init();

    while !terminated.load(Ordering::Relaxed) {
        let event = match receiver.try_recv() {
            Ok(event) => event,
            Err(_) => {
                thread::sleep(Duration::from_millis(TUI_EVENT_POLL_MS));
                continue;
            }
        };

        match &event {
            AppEvent::UiEvent(ui_event) => {
                if let Event::Key(key) = ui_event {
                    match key.code {
                        KeyCode::Char('x') => {
                            terminated.store(true, Ordering::Relaxed);
                            break;
                        }
                        KeyCode::Char('p') => {
                            let is_paused = paused.load(Ordering::Relaxed);
                            paused.store(!is_paused, Ordering::Relaxed);
                        }
                        KeyCode::Tab | KeyCode::Char('\t') => {
                            tab.next_tab(emulator_state_view.clone(), logging_view.clone())
                        }
                        _ => (),
                    }
                }
            }
            _ => (),
        }

        tab.handle_event(event);

        terminal
            .draw(|frame| {
                let [tabs_area, page_area] =
                    Layout::vertical([Constraint::Length(3), Constraint::Min(25)])
                        .areas(frame.area());

                Tabs::new(Tab::names().iter().cloned())
                    .block(Block::default().title("States").borders(Borders::ALL))
                    .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                    .select(tab.as_index())
                    .render(tabs_area, frame.buffer_mut());

                match tab {
                    Tab::EmulatorState(emulator_state_view) => {
                        emulator_state_view.draw(frame, page_area)
                    }
                    Tab::Logging(logging_view) => logging_view.draw(frame, page_area),
                }
            })
            .expect("Failed to draw TUI frame.");
    }

    ratatui::restore();
}

fn run_snapshot_thread(
    terminated: Arc<AtomicBool>,
    emulator: Arc<RwLock<EmulatorState>>,
    snapshot_sender: mpsc::Sender<AppEvent>,
) {
    while !terminated.load(Ordering::Relaxed) {
        let emulator = {
            let emulator = emulator.read().unwrap();
            emulator.clone()
        };

        let _ = snapshot_sender
            .send(AppEvent::StateEvent(emulator))
            .map_err(|_| log::error!("Error while submitting StateEvent with new snapshot."));

        thread::sleep(Duration::from_millis(SNAPSHOT_DELAY_MS));
    }
}

fn run_input_thread(terminated: Arc<AtomicBool>, event_sender: mpsc::Sender<AppEvent>) {
    trace!(target:"crossterm", "Starting input thread");

    while !terminated.load(Ordering::Relaxed) {
        if event::poll(Duration::from_millis(TUI_EVENT_POLL_MS)).unwrap() {
            let event = event::read().unwrap();
            trace!(target:"crossterm", "Stdin event received {:?}", event);
            event_sender.send(AppEvent::UiEvent(event)).unwrap();
        }
    }
}
