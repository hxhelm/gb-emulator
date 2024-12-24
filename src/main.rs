#![allow(clippy::upper_case_acronyms)]
use anyhow::{Context, Result};
use clap::Parser;
use emulator::Emulator;
use std::fs;
use std::path::PathBuf;
use tui::Debugger;

mod cpu;
mod emulator;
mod graphics;
mod memory;
mod tui;

const PATH_DMG_BOOT_ROM: &'static str = "./boot/dmg.bin";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The path to the game rom
    rom: PathBuf,

    /// Execute the boot rom instead of emulating the hand-off state
    #[arg(short = 'b', long)]
    boot: bool,

    /// Display the TUI debugger
    #[arg(short = 'd', long)]
    open_debugger: bool,

    /// Starts the emulator in paused state
    #[arg(short = 'p', long)]
    pause: bool,
}

fn main() -> Result<()> {
    color_eyre::install().unwrap();

    let cli = Cli::parse();

    let cartridge_contents = fs::read(cli.rom).context("Failed to read game rom.")?;

    let boot_contents = if cli.boot {
        let boot_rom = fs::read(PATH_DMG_BOOT_ROM).context("Failed to read binary rom.")?;
        Some(boot_rom)
    } else {
        None
    };

    let mut emulator = Emulator::init(boot_contents.as_deref(), &cartridge_contents, cli.pause)?;

    let debugger = if cli.open_debugger {
        Some(Debugger::new(
            emulator.state.clone(),
            emulator.terminated.clone(),
            emulator.paused.clone(),
        ))
    } else {
        None
    };

    emulator.start();

    if let Some(debugger) = debugger {
        debugger.snapshot_thread.join().unwrap();
        debugger.tui_thread.join().unwrap();
    }

    emulator.emulation_thread.join().unwrap();

    Ok(())
}
