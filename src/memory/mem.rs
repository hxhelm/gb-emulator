#![allow(unused)]
use crate::graphics::PPUMode;

pub const MEMORY_BUS_SIZE: usize = 0xFFFF + 1;
pub(crate) const ROM_BANK_0_START: u16 = 0x0000;
pub(crate) const ROM_BANK_0_END: u16 = 0x3FFF;
pub(crate) const ROM_BANK_1_START: u16 = 0x4000;
pub(crate) const ROM_BANK_1_END: u16 = 0x7FFF;
const VRAM_AREA_START: u16 = 0x8000;
const VRAM_AREA_END: u16 = 0x97FF;
const OAM_AREA_START: u16 = 0xFE00;
const OAM_AREA_END: u16 = 0xFE9F;

#[derive(Clone, Copy)]
pub struct Memory {
    memory: [u8; MEMORY_BUS_SIZE],
    pub ppu_mode: PPUMode,
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
    const fn address_is_accessible(&self, address: u16) -> bool {
        match address {
            VRAM_AREA_START..VRAM_AREA_END if matches!(self.ppu_mode, PPUMode::SendPixels) => false,
            OAM_AREA_START..OAM_AREA_END
                if matches!(self.ppu_mode, PPUMode::OBJSearch | PPUMode::SendPixels) =>
            {
                false
            }
            _ => true,
        }
    }

    pub(crate) const fn read(&self, address: u16) -> u8 {
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

    /// Ignores memory locations blocking by the PPU Mode, mainly used for the PPU itself
    pub(crate) const fn read_unchecked(&self, address: u16) -> u8 {
        // let permitted = match address {
        //     VRAM_AREA_START..VRAM_AREA_END => true,
        //     _ => false,
        // };
        //
        // assert!(permitted, "Unchecked memory access outside of VRAM area");

        self.memory[address as usize]
    }

    /// Used for debugging purposes only
    pub(crate) fn read_range(&self, address: u16, length: usize) -> &[u8] {
        let address = address as usize;
        let end = address + length;

        assert!(end <= MEMORY_BUS_SIZE);

        &self.memory[address..end]
    }
}
