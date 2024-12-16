use crate::memory::MEMORY_BUS_SIZE;
use crate::{cpu::CPU, emulator::EmulatorState};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};
use std::cmp::min;
use std::sync::RwLock;
use std::time;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

const MEMORY_VIEW_ELEMENTS_PER_LINE: usize = 16;
const MEMORY_VIEW_BLOCK_HEIGHT: usize = 18;
const MEMORY_SCROLL_MAX: usize =
    (MEMORY_BUS_SIZE / MEMORY_VIEW_ELEMENTS_PER_LINE).saturating_sub(MEMORY_VIEW_BLOCK_HEIGHT);
const MEMORY_BUFFER_SIZE: usize = MEMORY_VIEW_ELEMENTS_PER_LINE * MEMORY_VIEW_BLOCK_HEIGHT;

struct TUI {
    memory_vertical_scroll_state: ScrollbarState,
    memory_vertical_scroll: usize,
}

impl TUI {
    fn new() -> Self {
        Self {
            memory_vertical_scroll_state: ScrollbarState::new(0),
            memory_vertical_scroll: 0,
        }
    }

    fn draw(&mut self, frame: &mut Frame, cpu: &CPU) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(13),
                Constraint::Min(5),
                Constraint::Length(3),
            ])
            .split(frame.area());

        let instruction_widget =
            Paragraph::new(format!("Current Instruction: {:02X}", cpu.current_opcode))
                .block(Block::default().borders(Borders::ALL).title("Instruction"))
                .style(Style::default().fg(Color::Yellow));
        frame.render_widget(instruction_widget, chunks[0]);

        let register_items: Vec<ListItem> = self
            .register_view(cpu)
            .iter()
            .map(|reg| ListItem::new(reg.clone()))
            .collect();
        let registers_widget = List::new(register_items)
            .block(Block::default().borders(Borders::ALL).title("Registers"));
        frame.render_widget(registers_widget, chunks[1]);

        let scroll_status =
            Paragraph::new(format!("Scroll position: {}", self.memory_vertical_scroll)).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Scroll status"),
            );
        frame.render_widget(scroll_status, chunks[3]);

        let memory_row_offset = self.memory_vertical_scroll * MEMORY_VIEW_ELEMENTS_PER_LINE;
        let clamped_memory_row_offset =
            memory_row_offset.min(MEMORY_BUS_SIZE.saturating_sub(MEMORY_BUFFER_SIZE));
        let memory_range_end =
            (clamped_memory_row_offset + MEMORY_BUFFER_SIZE).min(MEMORY_BUS_SIZE);
        let memory_range_length = memory_range_end.saturating_sub(clamped_memory_row_offset);

        let memory_text = cpu
            .bus
            .read_range(clamped_memory_row_offset as u16, memory_range_length)
            .chunks(MEMORY_VIEW_ELEMENTS_PER_LINE)
            .enumerate()
            .map(|(i, chunk)| {
                Line::from(format!(
                    "{:04X} | {}",
                    clamped_memory_row_offset + i * MEMORY_VIEW_ELEMENTS_PER_LINE,
                    chunk
                        .iter()
                        .map(|&num| format!("{:02X}", num))
                        .collect::<Vec<_>>()
                        .join(" ")
                ))
            })
            .collect::<Vec<_>>();

        self.memory_vertical_scroll_state = self
            .memory_vertical_scroll_state
            .content_length(MEMORY_BUS_SIZE / MEMORY_VIEW_ELEMENTS_PER_LINE);

        let memory_view = Paragraph::new(memory_text)
            .block(Block::default().borders(Borders::ALL).title("Memory"));
        frame.render_widget(memory_view, chunks[2]);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            chunks[2],
            &mut self.memory_vertical_scroll_state,
        );
    }

    fn register_view(&self, cpu: &CPU) -> [String; 7] {
        [
            format!(
                "A: {:02X}  |  F: {:02X}",
                cpu.registers.a,
                u8::from(&cpu.registers.f)
            ),
            format!("B: {:02X}  |  C: {:02X}", cpu.registers.b, cpu.registers.c),
            format!("D: {:02X}  |  E: {:02X}", cpu.registers.d, cpu.registers.e),
            format!("H: {:02X}  |  L: {:02X}", cpu.registers.h, cpu.registers.l),
            format!("SP: {:04X}", cpu.registers.sp),
            format!("PC: {:04X}", cpu.registers.pc),
            format!(
                "Flags: {} {} {} {}",
                if cpu.registers.f.zero { 'Z' } else { '-' },
                if cpu.registers.f.negative { 'N' } else { '-' },
                if cpu.registers.f.carry { 'C' } else { '-' },
                if cpu.registers.f.half_carry { 'H' } else { '-' }
            ),
        ]
    }
}

pub struct Debugger {
    pub snapshot_thread: std::thread::JoinHandle<()>,
    pub tui_thread: std::thread::JoinHandle<()>,
}

impl Debugger {
    pub fn new(
        emulator: Arc<RwLock<EmulatorState>>,
        terminated: Arc<AtomicBool>,
        paused: Arc<AtomicBool>,
    ) -> Self {
        let emulator_snapshot = {
            let original = emulator.read().unwrap();
            Arc::new(Mutex::new(original.clone()))
        };

        let snapshot_thread = {
            let terminated_clone = Arc::clone(&terminated);
            let emulator_clone = Arc::clone(&emulator);
            let snapshot_clone = Arc::clone(&emulator_snapshot);

            thread::spawn(move || {
                while !terminated_clone.load(Ordering::Relaxed) {
                    {
                        let emulator = {
                            let emulator = emulator_clone.read().unwrap();
                            emulator
                        };

                        let mut snap = snapshot_clone.lock().unwrap();
                        *snap = *emulator;
                    }

                    thread::sleep(Duration::from_millis(16));
                }
            })
        };

        let tui_thread = {
            let mut terminal = ratatui::init();

            let tui = Arc::new(Mutex::new(TUI::new()));
            let emulator_clone = Arc::clone(&emulator);
            let paused_clone = Arc::clone(&paused);
            let snapshot_clone = Arc::clone(&emulator_snapshot);
            let terminated_clone = Arc::clone(&terminated);

            thread::spawn(move || {
                while !terminated_clone.load(Ordering::Relaxed) {
                    let mut tui_state = tui.lock().unwrap();

                    {
                        let emulator = snapshot_clone.lock().unwrap();

                        terminal
                            .draw(|frame| tui_state.draw(frame, &emulator.cpu))
                            .expect("failed to draw frame");
                    }

                    if event::poll(time::Duration::from_millis(5)).expect("Failed to poll events") {
                        if let Event::Key(key) = event::read().expect("Failed to read event") {
                            match key.code {
                                KeyCode::Char('x') => {
                                    terminated_clone.store(true, Ordering::Relaxed);
                                }
                                KeyCode::Char('j') | KeyCode::Down => {
                                    tui_state.memory_vertical_scroll = min(
                                        MEMORY_SCROLL_MAX,
                                        tui_state.memory_vertical_scroll.saturating_add(1),
                                    );
                                    tui_state.memory_vertical_scroll_state = tui_state
                                        .memory_vertical_scroll_state
                                        .position(tui_state.memory_vertical_scroll);
                                    drop(tui_state);
                                }
                                KeyCode::Char('k') | KeyCode::Up => {
                                    tui_state.memory_vertical_scroll =
                                        tui_state.memory_vertical_scroll.saturating_sub(1);
                                    tui_state.memory_vertical_scroll_state = tui_state
                                        .memory_vertical_scroll_state
                                        .position(tui_state.memory_vertical_scroll);
                                    drop(tui_state);
                                }
                                KeyCode::Char('d') => {
                                    tui_state.memory_vertical_scroll = min(
                                        MEMORY_SCROLL_MAX,
                                        tui_state
                                            .memory_vertical_scroll
                                            .saturating_add(MEMORY_VIEW_BLOCK_HEIGHT),
                                    );
                                    tui_state.memory_vertical_scroll_state = tui_state
                                        .memory_vertical_scroll_state
                                        .position(tui_state.memory_vertical_scroll);
                                    drop(tui_state);
                                }
                                KeyCode::Char('u') => {
                                    tui_state.memory_vertical_scroll = tui_state
                                        .memory_vertical_scroll
                                        .saturating_sub(MEMORY_VIEW_BLOCK_HEIGHT);
                                    tui_state.memory_vertical_scroll_state = tui_state
                                        .memory_vertical_scroll_state
                                        .position(tui_state.memory_vertical_scroll);
                                    drop(tui_state);
                                }
                                KeyCode::Char('n') => {
                                    let mut emulator = emulator_clone.write().unwrap();
                                    emulator.step();
                                }
                                KeyCode::Char('p') => {
                                    let is_paused = paused_clone.load(Ordering::Relaxed);
                                    paused_clone.store(!is_paused, Ordering::Relaxed);
                                }
                                _ => {}
                            }
                        }
                    }
                }

                ratatui::restore();
            })
        };

        Self {
            snapshot_thread,
            tui_thread,
        }
    }
}
