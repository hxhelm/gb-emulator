use crate::memory::bus::BUS_SIZE;
use crate::{cpu::CPU, emulator::EmulatorState};
use crossterm::event::{Event, KeyCode};
use ratatui::layout::Rect;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};
use std::{cmp::min, sync::Arc, sync::RwLock};

use super::{AppEvent, Page};

const MEMORY_VIEW_ELEMENTS_PER_LINE: usize = 16;

#[derive(Clone)]
pub(super) struct EmulatorStateView {
    memory_vertical_scroll_state: ScrollbarState,
    memory_vertical_scroll: usize,
    memory_vertical_scroll_max: usize,
    memory_view_block_height: usize,
    emulator_snapshot: Arc<RwLock<EmulatorState>>,
}

impl EmulatorStateView {
    pub(super) fn new(emulator_state: Arc<RwLock<EmulatorState>>) -> Self {
        Self {
            memory_vertical_scroll_state: ScrollbarState::new(0),
            memory_vertical_scroll: 0,
            memory_vertical_scroll_max: 0,
            memory_view_block_height: 0,
            emulator_snapshot: emulator_state,
        }
    }
}

impl Page for EmulatorStateView {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let cpu = &self.emulator_snapshot.read().unwrap().cpu;

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(13),
                Constraint::Min(5),
                Constraint::Length(3),
            ])
            .split(area);

        let instruction_widget = Paragraph::new(format!(
            "Current Instruction: {:02X}",
            cpu.current_instruction.opcode
        ))
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

        let memory_chunk = chunks[2];
        self.memory_view_block_height = (memory_chunk.height - 2).into();
        let memory_buffer_size: usize =
            MEMORY_VIEW_ELEMENTS_PER_LINE * self.memory_view_block_height;

        let memory_row_offset = self.memory_vertical_scroll * MEMORY_VIEW_ELEMENTS_PER_LINE;
        let clamped_memory_row_offset =
            memory_row_offset.min(BUS_SIZE.saturating_sub(memory_buffer_size));
        let memory_range_end = (clamped_memory_row_offset + memory_buffer_size).min(BUS_SIZE);
        let memory_range_length = memory_range_end.saturating_sub(clamped_memory_row_offset);

        let memory_text = cpu
            .bus
            .read_range_debug(clamped_memory_row_offset as u16, memory_range_length as u16)
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
            .content_length(BUS_SIZE / MEMORY_VIEW_ELEMENTS_PER_LINE);
        self.memory_vertical_scroll_max = (BUS_SIZE / MEMORY_VIEW_ELEMENTS_PER_LINE)
            .saturating_sub(self.memory_view_block_height);

        let memory_view = Paragraph::new(memory_text)
            .block(Block::default().borders(Borders::ALL).title("Memory"));
        frame.render_widget(memory_view, memory_chunk);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            chunks[2],
            &mut self.memory_vertical_scroll_state,
        );
    }

    fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::UiEvent(ui_event) => {
                if let Event::Key(key) = ui_event {
                    match key.code {
                        KeyCode::Char('j') | KeyCode::Down => {
                            self.memory_vertical_scroll = min(
                                self.memory_vertical_scroll_max,
                                self.memory_vertical_scroll.saturating_add(1),
                            );
                            self.memory_vertical_scroll_state = self
                                .memory_vertical_scroll_state
                                .position(self.memory_vertical_scroll);
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            self.memory_vertical_scroll =
                                self.memory_vertical_scroll.saturating_sub(1);
                            self.memory_vertical_scroll_state = self
                                .memory_vertical_scroll_state
                                .position(self.memory_vertical_scroll);
                        }
                        KeyCode::Char('d') => {
                            self.memory_vertical_scroll = min(
                                self.memory_vertical_scroll_max,
                                self.memory_vertical_scroll
                                    .saturating_add(self.memory_view_block_height),
                            );
                            self.memory_vertical_scroll_state = self
                                .memory_vertical_scroll_state
                                .position(self.memory_vertical_scroll);
                        }
                        KeyCode::Char('u') => {
                            self.memory_vertical_scroll = self
                                .memory_vertical_scroll
                                .saturating_sub(self.memory_view_block_height);
                            self.memory_vertical_scroll_state = self
                                .memory_vertical_scroll_state
                                .position(self.memory_vertical_scroll);
                        }
                        KeyCode::Char('n') => {
                            let mut emulator = self.emulator_snapshot.write().unwrap();
                            emulator.step();
                        }
                        _ => {}
                    }
                }
            }
            AppEvent::StateEvent(emulator_state) => {
                let mut emulator_snapshot = self.emulator_snapshot.write().unwrap();
                *emulator_snapshot = emulator_state;
            }
        }
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
