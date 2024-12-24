use super::mem::{Memory, ROM_BANK_0_START, ROM_BANK_1_END};
use crate::graphics::PPUMode;

const BOOT_DISABLE: u16 = 0xFF50;
const BOOT_ROM_LENGTH: u16 = 0x0100;

#[derive(Clone, Copy)]
pub struct Bus {
    memory: Memory,
    boot_rom: [u8; BOOT_ROM_LENGTH as usize],
    pub(crate) boot_rom_disabled: bool,
}

impl Default for Bus {
    fn default() -> Self {
        Self {
            memory: Memory::default(),
            boot_rom: [0; BOOT_ROM_LENGTH as usize],
            boot_rom_disabled: false,
        }
    }
}

impl Bus {
    pub const fn read_byte(&self, address: u16) -> u8 {
        if self.boot_rom_disabled {
            self.memory.read(address)
        } else {
            match address {
                0..BOOT_ROM_LENGTH => self.boot_rom[address as usize],
                _ => self.memory.read(address),
            }
        }
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            BOOT_DISABLE => {
                self.boot_rom_disabled = byte != 0;
                self.memory.write(address, byte);
            }
            _ => self.memory.write(address, byte),
        }
    }

    /// Ignores memory locations blocking by the PPU Mode, mainly used for the PPU itself
    pub const fn read_byte_unchecked(&self, address: u16) -> u8 {
        self.memory.read_unchecked(address)
    }

    pub fn read_byte_at_offset(&self, offset: u8) -> u8 {
        let address = 0xFF00 + u16::from(offset);
        self.read_byte(address)
    }

    pub fn write_byte_at_offset(&mut self, offset: u8, byte: u8) {
        let address = 0xFF00 + u16::from(offset);
        self.write_byte(address, byte);
    }

    pub fn write_word(&mut self, address: u16, word: u16) {
        let [lsb, msb] = word.to_le_bytes();
        self.write_byte(address, lsb);
        self.write_byte(address + 1, msb);
    }

    pub const fn read_word(&self, address: u16) -> u16 {
        let lsb = self.read_byte(address);
        let msb = self.read_byte(address + 1);

        u16::from_le_bytes([lsb, msb])
    }

    pub fn update_ppu_mode(&mut self, mode: PPUMode) {
        self.memory.ppu_mode = mode;
    }

    /// Used for debugging purposes only, wrapper `Memory::read_range`
    pub fn read_range(&self, address: u16, length: usize) -> Vec<u8> {
        if self.boot_rom_disabled {
            return Vec::from(self.memory.read_range(address, length));
        }

        let mut result = Vec::new();
        let start = address as usize;
        let end = start + length;
        let boot_rom_end = BOOT_ROM_LENGTH as usize;

        if start < boot_rom_end {
            let boot_start = start;
            let boot_end = end.min(boot_rom_end);
            result.extend_from_slice(&self.boot_rom[boot_start..boot_end]);
        }

        if end > boot_rom_end {
            let mem_start = start.max(boot_rom_end);
            let mem_end = end;
            result.extend_from_slice(
                self.memory
                    .read_range(mem_start as u16, mem_end - mem_start),
            );
        }

        assert_eq!(result.len(), length);

        result
    }

    pub fn write_cartridge(&mut self, rom: &[u8]) {
        let start: usize = ROM_BANK_0_START.into();
        let end: usize = ROM_BANK_1_END.into();

        let slice = &rom[start..end.min(rom.len())];

        for (i, byte) in slice.iter().enumerate() {
            self.memory.write(i as u16, *byte);
        }
    }

    pub fn write_boot_rom(&mut self, rom: &[u8]) {
        let boot_rom_length: usize = BOOT_ROM_LENGTH.into();
        assert!(
            rom.len() >= boot_rom_length,
            "Boot ROM expected to be {} Bytes long",
            boot_rom_length
        );

        self.boot_rom.copy_from_slice(&rom[..boot_rom_length]);
    }
}

pub fn get_bit_status(byte: u8, position: u8) -> bool {
    byte & (1 << position) != 0
}

pub fn set_bit_status(byte: u8, position: u8, status: bool) -> u8 {
    if status {
        byte | 1 << position
    } else {
        byte & !(1 << position)
    }
}
