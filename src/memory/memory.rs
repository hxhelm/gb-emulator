use std::usize;

use crate::graphics::PPUMode;

pub(crate) const MEMORY_BUS_SIZE: usize = 0xFFFF;
pub const VRAM_AREA_START: u16 = 0x8000;
pub const VRAM_AREA_END: u16 = 0x97FF;
pub const _VRAM_TILE_SIZE: u16 = 16;
pub const OAM_AREA_START: u16 = 0xFE00;
pub const OAM_AREA_END: u16 = 0xFE9F;

#[derive(Clone, Copy)]
pub(crate) struct Memory {
    memory: [u8; MEMORY_BUS_SIZE],
    pub(crate) ppu_mode: PPUMode,
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            memory: [Default::default(); MEMORY_BUS_SIZE],
            ppu_mode: PPUMode::VerticalBlank,
        }
    }
}

impl Memory {
    fn address_is_accessible(&self, address: u16) -> bool {
        match address {
            VRAM_AREA_START..VRAM_AREA_END if matches!(self.ppu_mode, PPUMode::SendPixels) => false,
            OAM_AREA_START..OAM_AREA_END => match self.ppu_mode {
                PPUMode::OBJSearch | PPUMode::SendPixels => false,
                _ => true,
            },
            _ => true,
        }
    }

    pub(crate) fn read(&self, address: u16) -> u8 {
        if self.address_is_accessible(address) {
            self.memory[address as usize]
        } else {
            0xFF
        }
    }

    pub(crate) fn write(&mut self, address: u16, byte: u8) {
        if self.address_is_accessible(address) {
            self.memory[address as usize] = byte;
        }
    }

    /// Used for debugging purposes only
    pub(crate) fn read_range(&self, address: u16, length: usize) -> &[u8] {
        let address = address as usize;
        let end = address + length;

        assert!(end < MEMORY_BUS_SIZE);

        &self.memory[address..end]
    }
}
