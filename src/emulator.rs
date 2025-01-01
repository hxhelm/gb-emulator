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
    time::{Duration, Instant},
};

#[derive(Clone)]
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

    pub fn step(&mut self) -> u8 {
        let cycles = self.cpu.step();

        self.framebuffer = self.ppu.step(cycles, &mut self.cpu.bus);

        cycles
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
    pub fn init(
        boot_contents: Option<&[u8]>,
        cartridge_contents: &[u8],
        paused: bool,
    ) -> Result<Self> {
        let cpu = CPU::init(boot_contents, cartridge_contents);

        let state = Arc::new(RwLock::new(EmulatorState::init(cpu)));
        let terminated = Arc::new(AtomicBool::new(false));
        let paused = Arc::new(AtomicBool::new(paused));

        let (frame_sender, frame_receiver) = bounded(3);

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

const CLOCK_SPEED: u32 = 4_194_304;
const FRAME_RATE: u32 = 60;
const CYCLES_PER_FRAME: u32 = CLOCK_SPEED / FRAME_RATE;

fn start_emulation(
    state: Arc<RwLock<EmulatorState>>,
    terminated: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    frame_sender: Sender<PixelData>,
) -> JoinHandle<()> {
    let state = state.clone();
    let paused_clone = Arc::clone(&paused);
    let terminated_clone = Arc::clone(&terminated);
    let frame_duration = Duration::from_secs_f64(1.0 / FRAME_RATE as f64);

    thread::spawn(move || {
        while !terminated_clone.load(Ordering::Relaxed) {
            let frame_start = Instant::now();
            let mut cycles_this_frame: u32 = 0;
            let mut frame_drawn = false;

            if !paused_clone.load(Ordering::Relaxed) {
                let mut emulator = state.write().unwrap();
                while cycles_this_frame < CYCLES_PER_FRAME {
                    let cycles = emulator.step();
                    cycles_this_frame += cycles as u32;

                    if emulator.framebuffer.is_some() {
                        frame_drawn = true;
                        break;
                    }
                }
            }

            if frame_drawn {
                if let Some(framebuffer) = state.read().unwrap().framebuffer {
                    frame_sender.send(framebuffer).unwrap();
                }
            }

            let elapsed = frame_start.elapsed();

            if elapsed < frame_duration {
                thread::sleep(frame_duration - elapsed);
            } else {
                eprintln!("Frame took too long: {:?}ms", elapsed.as_millis());
            }
        }
    })
}
