use crate::cpu::CPU;
use crate::graphics::PPU;
use anyhow::Result;
use std::fs;

const PATH_DMG_BOOT_ROM: &'static str = "./boot/dmg.bin";

#[derive(Clone)]
pub struct Emulator {
    pub cpu: CPU,
    pub ppu: PPU,
}

impl Emulator {
    // TODO: pass rom data as a parameter to this function, possibly taken as cli argument
    pub fn init() -> Result<Self> {
        let boot_rom_path = PATH_DMG_BOOT_ROM;
        let boot_rom = fs::read(boot_rom_path)?;

        let mut cpu = CPU::default();
        let ppu = PPU::init();

        cpu.boot_rom(boot_rom.as_slice());

        Ok(Emulator { cpu, ppu })
    }

    pub fn step(&mut self) {
        let cycles = self.cpu.step();
        self.ppu.step(cycles, &mut self.cpu.bus);
    }
}
