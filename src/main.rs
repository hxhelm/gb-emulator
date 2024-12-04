#![allow(clippy::upper_case_acronyms)]
use anyhow::Result;
use cpu::CPU;
use crossterm::event::{self, Event, KeyCode};
use emulator::Emulator;
use memory::MEMORY_BUS_SIZE;
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
use std::time;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

mod cpu;
mod emulator;
mod graphics;
mod memory;

const MEMORY_VIEW_ELEMENTS_PER_LINE: usize = 16;
const MEMORY_VIEW_BLOCK_HEIGHT: usize = 20;
const MEMORY_SCROLL_MAX: usize =
    MEMORY_BUS_SIZE / MEMORY_VIEW_ELEMENTS_PER_LINE - MEMORY_VIEW_BLOCK_HEIGHT;

struct Debugger {
    pub memory_vertical_scroll_state: ScrollbarState,
    pub memory_vertical_scroll: usize,
}

impl Debugger {
    const fn new() -> Self {
        Self {
            memory_vertical_scroll_state: ScrollbarState::new(0),
            memory_vertical_scroll: 0,
        }
    }

    fn draw(&mut self, frame: &mut Frame, emulator: &Emulator) {
        let cpu = &emulator.cpu;

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

        let register_items: Vec<ListItem> = register_view(cpu)
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

        let memory_buffer_size = MEMORY_VIEW_ELEMENTS_PER_LINE * MEMORY_VIEW_BLOCK_HEIGHT;
        let memory_row_offset = min(
            MEMORY_BUS_SIZE - memory_buffer_size - 1,
            self.memory_vertical_scroll * MEMORY_VIEW_ELEMENTS_PER_LINE,
        );
        let memory_text = cpu
            .bus
            .read_range(memory_row_offset as u16, memory_buffer_size)
            .chunks(MEMORY_VIEW_ELEMENTS_PER_LINE)
            .enumerate()
            .map(|(i, chunk)| {
                Line::from(format!(
                    "{:04X} | {}",
                    memory_row_offset + i * MEMORY_VIEW_ELEMENTS_PER_LINE,
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
}

fn register_view(cpu: &CPU) -> [String; 7] {
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

#[allow(clippy::too_many_lines)]
fn main() -> Result<()> {
    color_eyre::install().unwrap();

    let emulator_init = Emulator::init()?;
    let emulator = Arc::new(Mutex::new(emulator_init.clone()));
    let emulator_snapshot = Arc::new(Mutex::new(emulator_init.clone()));

    let paused = Arc::new(AtomicBool::new(true));
    let terminate = Arc::new(AtomicBool::new(false));

    let cpu_thread = {
        let emulator_clone = Arc::clone(&emulator);
        let paused_clone = Arc::clone(&paused);
        let terminate_clone = Arc::clone(&terminate);

        thread::spawn(move || {
            while !terminate_clone.load(Ordering::Relaxed) {
                if !paused_clone.load(Ordering::Relaxed) {
                    let mut emulator = emulator_clone.lock().unwrap();

                    if !emulator.cpu.is_halted {
                        emulator.step();
                    }
                }

                thread::sleep(Duration::from_nanos(10));
            }
        })
    };

    let snapshot_thread = {
        let terminate_clone = Arc::clone(&terminate);
        let emulator_clone = Arc::clone(&emulator);
        let snapshot_clone = Arc::clone(&emulator_snapshot);

        thread::spawn(move || {
            while !terminate_clone.load(Ordering::Relaxed) {
                {
                    let emulator = {
                        let emulator = emulator_clone.lock().unwrap();
                        emulator
                    };

                    let mut snap = snapshot_clone.lock().unwrap();
                    *snap = emulator.clone();
                }

                thread::sleep(Duration::from_millis(16));
            }
        })
    };

    let tui_thread = {
        let mut terminal = ratatui::init();

        let debugger = Arc::new(Mutex::new(Debugger::new()));
        let emulator_clone = Arc::clone(&emulator);
        let paused_clone = Arc::clone(&paused);
        let snapshot_clone = Arc::clone(&emulator_snapshot);
        let terminate_clone = Arc::clone(&terminate);

        thread::spawn(move || loop {
            let mut debugger_state = debugger.lock().unwrap();

            {
                let emulator = snapshot_clone.lock().unwrap();

                terminal
                    .draw(|frame| debugger_state.draw(frame, &emulator))
                    .expect("failed to draw frame");
            }

            if event::poll(time::Duration::from_millis(5)).expect("Failed to poll events") {
                if let Event::Key(key) = event::read().expect("Failed to read event") {
                    match key.code {
                        KeyCode::Char('x') => {
                            terminate_clone.store(true, Ordering::Relaxed);
                            ratatui::restore();
                            break;
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            debugger_state.memory_vertical_scroll = min(
                                MEMORY_SCROLL_MAX,
                                debugger_state.memory_vertical_scroll.saturating_add(1),
                            );
                            debugger_state.memory_vertical_scroll_state = debugger_state
                                .memory_vertical_scroll_state
                                .position(debugger_state.memory_vertical_scroll);
                            drop(debugger_state);
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            debugger_state.memory_vertical_scroll =
                                debugger_state.memory_vertical_scroll.saturating_sub(1);
                            debugger_state.memory_vertical_scroll_state = debugger_state
                                .memory_vertical_scroll_state
                                .position(debugger_state.memory_vertical_scroll);
                            drop(debugger_state);
                        }
                        KeyCode::Char('d') => {
                            debugger_state.memory_vertical_scroll = min(
                                MEMORY_SCROLL_MAX,
                                debugger_state
                                    .memory_vertical_scroll
                                    .saturating_add(MEMORY_VIEW_BLOCK_HEIGHT),
                            );
                            debugger_state.memory_vertical_scroll_state = debugger_state
                                .memory_vertical_scroll_state
                                .position(debugger_state.memory_vertical_scroll);
                            drop(debugger_state);
                        }
                        KeyCode::Char('u') => {
                            debugger_state.memory_vertical_scroll = debugger_state
                                .memory_vertical_scroll
                                .saturating_sub(MEMORY_VIEW_BLOCK_HEIGHT);
                            debugger_state.memory_vertical_scroll_state = debugger_state
                                .memory_vertical_scroll_state
                                .position(debugger_state.memory_vertical_scroll);
                            drop(debugger_state);
                        }
                        KeyCode::Char('n') => {
                            let mut emulator = emulator_clone.lock().unwrap();

                            if !emulator.cpu.is_halted {
                                emulator.step();
                            }
                        }
                        KeyCode::Char('p') => {
                            let is_paused = paused_clone.load(Ordering::Relaxed);
                            paused_clone.store(!is_paused, Ordering::Relaxed);
                        }
                        _ => {}
                    }
                }
            }
        })
    };

    tui_thread.join().unwrap();
    snapshot_thread.join().unwrap();
    cpu_thread.join().unwrap();

    Ok(())
}
