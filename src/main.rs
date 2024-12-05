#![allow(clippy::upper_case_acronyms)]
use anyhow::Result;
use emulator::Emulator;
use tui::Debugger;

mod cpu;
mod emulator;
mod graphics;
mod memory;
mod tui;

fn main() -> Result<()> {
    color_eyre::install().unwrap();

    let mut emulator = Emulator::init()?;
    let debugger = Debugger::new(
        emulator.state.clone(),
        emulator.terminated.clone(),
        emulator.paused.clone(),
    );

    emulator.start();

    debugger.snapshot_thread.join().unwrap();
    debugger.tui_thread.join().unwrap();
    emulator.emulation_thread.join().unwrap();

    Ok(())
}
