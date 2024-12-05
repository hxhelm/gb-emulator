#![allow(unused)]
use crate::cpu::CPU;
use crate::graphics::{App, PixelData, LCD_HEIGHT, LCD_WIDTH, PPU};
use anyhow::Result;
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
    framebuffer: PixelData,
}

impl EmulatorState {
    pub fn step(&mut self) {
        let cycles = self.cpu.step();
        self.ppu.step(cycles, &mut self.cpu.bus);
    }
}

pub struct Emulator {
    pub state: Arc<RwLock<EmulatorState>>,
    pub emulation_thread: JoinHandle<()>,
    pub terminated: Arc<AtomicBool>,
    pub paused: Arc<AtomicBool>,
}

impl Emulator {
    // TODO: pass rom data as a parameter to this function, possibly taken as cli argument
    pub fn init() -> Result<Self> {
        let boot_rom_path = PATH_DMG_BOOT_ROM;
        let boot_rom = fs::read(boot_rom_path)?;

        let mut cpu = CPU::default();
        cpu.boot_rom(boot_rom.as_slice());

        let ppu = PPU::init();

        let framebuffer = [0; LCD_WIDTH * LCD_HEIGHT];
        let state = Arc::new(RwLock::new(EmulatorState {
            cpu,
            ppu,
            framebuffer,
        }));

        let terminated = Arc::new(AtomicBool::new(false));
        let paused = Arc::new(AtomicBool::new(true));
        let emulation_thread = start_emulation(state.clone(), terminated.clone(), paused.clone());

        let emulator = Emulator {
            state,
            emulation_thread,
            terminated,
            paused,
        };

        Ok(emulator)
    }

    pub fn start(&mut self) {
        App::init(self.terminated.clone());
    }
}

fn start_emulation(
    state: Arc<RwLock<EmulatorState>>,
    terminated: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
) -> JoinHandle<()> {
    let state = state.clone();
    let paused_clone = Arc::clone(&paused);
    let terminated_clone = Arc::clone(&terminated);

    thread::spawn(move || {
        while !terminated_clone.load(Ordering::Relaxed) {
            if !paused_clone.load(Ordering::Relaxed) {
                let mut emulator = state.write().unwrap();

                if !emulator.cpu.is_halted {
                    emulator.step();
                }
            }

            thread::sleep(Duration::from_nanos(10));
        }
    })
}
