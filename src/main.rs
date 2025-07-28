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

const PATH_DMG_BOOT_ROM: &str = "./boot/dmg.bin";

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

fn init_logging(use_tui_debugger: bool) {
    use log::LevelFilter;
    use tui_logger::{init_logger, set_default_level};

    if use_tui_debugger {
        init_logger(LevelFilter::Info).unwrap();
        set_default_level(LevelFilter::Info);
    } else {
        env_logger::init();
    }
}

fn main() -> Result<()> {
    color_eyre::install().unwrap();

    let cli = Cli::parse();

    init_logging(cli.open_debugger);

    let cartridge_contents = fs::read(cli.rom).context("Failed to read game rom.")?;

    let boot_contents = cli
        .boot
        .then(|| fs::read(PATH_DMG_BOOT_ROM).context("Failed to read binary rom."))
        .transpose()?;

    let mut emulator = Emulator::init(boot_contents.as_deref(), &cartridge_contents, cli.pause)?;

    let debugger = cli
        .open_debugger
        .then(|| Debugger::new(&emulator.state, &emulator.terminated, &emulator.paused));

    // start main emulation loop
    emulator.start();

    debugger.map(|d| d.shutdown());

    emulator.emulation_thread.join().unwrap();

    Ok(())
}
