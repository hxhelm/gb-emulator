use crate::{graphics::PPUMode, memory::memory::Memory};

#[derive(Clone, Copy, Default)]
pub(crate) struct Bus {
    memory: Memory,
}

impl Bus {
    pub fn read_byte(&self, address: u16) -> u8 {
        self.memory.read(address)
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        self.memory.write(address, byte);
    }

    pub fn read_byte_at_offset(&self, offset: u8) -> u8 {
        let address = 0xFF00 + offset as u16;
        self.read_byte(address)
    }

    pub fn write_byte_at_offset(&mut self, offset: u8, byte: u8) {
        let address = 0xFF00 + offset as u16;
        self.write_byte(address, byte)
    }

    pub fn write_word(&mut self, address: u16, word: u16) {
        let [lsb, msb] = word.to_le_bytes();
        self.memory.write(address, lsb);
        self.memory.write(address + 1, msb);
    }

    pub fn read_word(&mut self, address: u16) -> u16 {
        let lsb = self.memory.read(address);
        let msb = self.memory.read(address + 1);

        u16::from_le_bytes([lsb, msb])
    }

    pub fn update_ppu_mode(&mut self, mode: &PPUMode) {
        self.memory.ppu_mode = mode.clone();
    }

    /// Used for debugging purposes only, wrapper Memory::read_range
    pub fn read_range(&self, address: u16, length: usize) -> &[u8] {
        self.memory.read_range(address, length)
    }
}