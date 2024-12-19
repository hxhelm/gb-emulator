#![allow(unused)]
use crate::cpu::CPU;
use crate::graphics::{App, PixelData, LCD_HEIGHT, LCD_WIDTH, PPU};
use anyhow::Result;
use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use std::fs;
use std::thread::JoinHandle;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    sync::{Arc, Mutex, RwLock},
    thread,
    time::Duration,
};

const PATH_DMG_BOOT_ROM: &'static str = "./boot/dmg.bin";

#[derive(Clone, Copy)]
pub struct EmulatorState {
    pub cpu: CPU,
    pub ppu: PPU,
    pub framebuffer: Option<PixelData>,
}

impl EmulatorState {
    pub fn init(cpu: CPU) -> Self {
        Self {
            cpu,
            ppu: PPU::init(),
            framebuffer: None,
        }
    }

    pub fn step(&mut self) {
        if self.cpu.is_halted {
            return;
        }

        let cycles = self.cpu.step();

        self.framebuffer = self.ppu.step(cycles, &mut self.cpu.bus);
    }
}

pub struct Emulator {
    /// Transferable Emulator State
    pub state: Arc<RwLock<EmulatorState>>,
    /// Emulation thread & synchronization flags
    pub emulation_thread: JoinHandle<()>,
    pub terminated: Arc<AtomicBool>,
    pub paused: Arc<AtomicBool>,
    /// Rendering
    app: App,
}

impl Emulator {
    pub fn init(cartridge_contents: Option<&[u8]>) -> Result<Self> {
        let mut cpu = CPU::default();

        if let Some(rom) = cartridge_contents {
            cpu.load_cartridge(rom);
        }

        let boot_rom_path = PATH_DMG_BOOT_ROM;
        let boot_rom = fs::read(boot_rom_path)?;

        cpu.load_boot_rom(boot_rom.as_slice());

        let state = Arc::new(RwLock::new(EmulatorState::init(cpu)));
        let terminated = Arc::new(AtomicBool::new(false));
        let paused = Arc::new(AtomicBool::new(true));

        let (frame_sender, frame_receiver) = bounded(10);

        let emulation_thread = start_emulation(
            state.clone(),
            terminated.clone(),
            paused.clone(),
            frame_sender,
        );

        let app = App::init(terminated.clone(), frame_receiver);

        let emulator = Emulator {
            app,
            state,
            emulation_thread,
            terminated,
            paused,
        };

        Ok(emulator)
    }

    pub fn start(&mut self) {
        self.app.run();
    }
}

fn start_emulation(
    state: Arc<RwLock<EmulatorState>>,
    terminated: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    frame_sender: Sender<PixelData>,
) -> JoinHandle<()> {
    let state = state.clone();
    let paused_clone = Arc::clone(&paused);
    let terminated_clone = Arc::clone(&terminated);

    thread::spawn(move || {
        while !terminated_clone.load(Ordering::Relaxed) {
            if !paused_clone.load(Ordering::Relaxed) {
                let mut emulator = state.write().unwrap();
                emulator.step();
            }

            let emulator = state.read().unwrap();
            if let Some(framebuffer) = emulator.framebuffer {
                frame_sender.send(framebuffer).unwrap();
            }

            thread::sleep(Duration::from_nanos(100));
        }
    })
}
