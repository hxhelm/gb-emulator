use crate::graphics::PPU;
use crate::CPU;

struct Emulator {
    cpu: CPU,
    ppu: PPU,
}

impl Emulator {
    pub fn step(&mut self) {
        let cycles = self.cpu.step();
        self.ppu.step(cycles);
    }
}
