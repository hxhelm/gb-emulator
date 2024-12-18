#![allow(clippy::upper_case_acronyms)]
use anyhow::Result;
use emulator::Emulator;
use std::{env, fs};
use tui::Debugger;

mod cpu;
mod emulator;
mod graphics;
mod memory;
mod tui;

fn main() -> Result<()> {
    color_eyre::install().unwrap();

    let args: Vec<String> = env::args().collect();

    let cartridge_contents = args
        .get(1)
        .map(|bin_path| fs::read(bin_path).expect("Failed to cartridge binary path."));

    let mut emulator = Emulator::init(cartridge_contents.as_deref())?;
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
