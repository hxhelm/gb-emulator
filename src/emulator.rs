use crate::cpu::CPU;
use crate::graphics::PPU;

#[derive(Clone, Copy)]
pub struct Emulator {
    pub cpu: CPU,
    pub ppu: PPU,
}

impl Emulator {
    pub fn step(&mut self) {
        let cycles = self.cpu.step();
        self.ppu.step(cycles, &mut self.cpu.bus);
    }
}
