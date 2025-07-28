#![allow(unused)]

use super::bus::{BYTE_INVALID_READ, CARTRIDGE_RAM_SIZE, CARTRIDGE_ROM_SIZE};

const RAM_ENABLE_END: u16 = 0x1FFF;
const RAM_ENABLE_VALUE: u8 = 0x0A;
const ROM_BANK_NUMBER_START: u16 = 0x2000;
const ROM_BANK_NUMBER_END: u16 = 0x3FFF;
const RAM_BANK_NUMBER_START: u16 = 0x4000;
const RAM_BANK_NUMBER_END: u16 = 0x5FFF;

#[derive(Clone)]
pub struct Memory {
    rom: Vec<u8>,
    rom_size: RomSize,
    rom_bank: u16,
    ram: Vec<u8>,
    ram_size: RamSize,
    ram_bank: u16,
    ram_enabled: bool,
}

#[derive(Clone)]
enum RomSize {
    Unset,
    Extended(u32, u16),
}

const ROM_BANK_SIZE: u16 = 0x4000; // 16 KiB
const ROM_DEFAULT_SIZE: u32 = 0x8000; // 32 KiB

impl RomSize {
    fn from_header(value: u8) -> Self {
        match value {
            0 => Self::Unset,
            1..8 => Self::Extended(ROM_DEFAULT_SIZE * (1 << value), 1 << (value + 1)),
            _ => panic!(
                "Invalid cartridge header at [0148](ROM Size): 0x{:02X}",
                value
            ),
        }
    }

    fn bytes(&self) -> u32 {
        match self {
            Self::Unset => ROM_DEFAULT_SIZE,
            Self::Extended(size, _) => *size,
        }
    }

    fn banks(&self) -> u16 {
        match self {
            Self::Unset => 0,
            Self::Extended(_, banks) => *banks,
        }
    }
}

const RAM_BANK_SIZE: u16 = 0x2000; // 8 KiB

#[derive(Clone)]
enum RamSize {
    Unset,
    Extended(u32, u16),
}

impl RamSize {
    fn from_header(value: u8) -> Self {
        match value {
            0 => Self::Unset,
            2 => Self::Extended(RAM_BANK_SIZE.into(), 1),
            3 => Self::Extended((RAM_BANK_SIZE << 2).into(), 4),
            4 => Self::Extended((RAM_BANK_SIZE << 4).into(), 16),
            5 => Self::Extended((RAM_BANK_SIZE << 3).into(), 8),
            _ => panic!(
                "Invalid cartridge header at [0149](RAM Size): 0x{:02X}",
                value
            ),
        }
    }

    fn bytes(&self) -> u32 {
        match self {
            Self::Unset => 0,
            Self::Extended(size, _) => *size,
        }
    }

    fn banks(&self) -> u16 {
        match self {
            Self::Unset => 0,
            Self::Extended(_, banks) => *banks,
        }
    }
}

// load the cartridge contents into rom. Needed since Memory::write ignores writes into the ROM
// section
fn write_cartridge_to_rom(rom: &mut [u8], cartridge_contents: &[u8]) {
    let rom_size = rom.len();

    if cartridge_contents.len() != rom_size {
        log::warn!(
            "Requested ROM size of 0x{:02X}B, but got cartridge of size 0x{:02X}B instead",
            rom_size,
            cartridge_contents.len()
        );
    }

    let slice = &cartridge_contents[0..rom_size.min(cartridge_contents.len())];

    for (i, byte) in slice.iter().enumerate() {
        rom[i as usize] = *byte;
    }
}

impl Memory {
    /// Allocate a single Vec for the cartridge rom and ram
    pub(super) fn init(cartridge_contents: &[u8]) -> Self {
        let rom_size = RomSize::from_header(cartridge_contents[usize::from(CARTRIDGE_ROM_SIZE)]);
        let ram_size = RamSize::from_header(cartridge_contents[usize::from(CARTRIDGE_RAM_SIZE)]);

        let mut rom = vec![BYTE_INVALID_READ; usize::try_from(rom_size.bytes()).unwrap()];

        write_cartridge_to_rom(&mut rom, cartridge_contents);

        Self {
            rom,
            rom_size,
            rom_bank: 1,
            ram: vec![BYTE_INVALID_READ; usize::try_from(ram_size.bytes()).unwrap()],
            ram_size,
            ram_bank: 0,
            ram_enabled: false,
        }
    }

    pub(super) fn read_rom(&self, address: u16) -> u8 {
        match self.rom_size {
            RomSize::Unset => self.rom[address as usize],
            RomSize::Extended(_, _) => match address {
                0x0000..=0x3FFF => self.rom[address as usize],
                0x4000..=0x7FFF => {
                    let index = usize::from(address - ROM_BANK_SIZE)
                        + (usize::from(self.rom_bank) * usize::from(ROM_BANK_SIZE));

                    assert!(index < self.rom.len());

                    self.rom[index]
                }
                _ => BYTE_INVALID_READ,
            },
        }
    }

    // TODO: https://gbdev.io/pandocs/MBC1.html#60007fff--banking-mode-select-write-only
    /// Triggers state changes when specific ROM addresses are written to. Never writes into actual
    /// ROM sections!
    pub(super) fn write_rom(&mut self, address: u16, byte: u8) {
        match address {
            0..=RAM_ENABLE_END => {
                self.ram_enabled = (byte & 0x0F) == RAM_ENABLE_VALUE;
            }
            ROM_BANK_NUMBER_START..=ROM_BANK_NUMBER_END => {
                let bank_mask = (self.rom_size.banks().saturating_sub(1));
                let bank_number = u16::from(byte) & (0x1F & bank_mask);

                // bank 0 is not valid
                self.rom_bank = if bank_number == 0 { 1 } else { bank_number };
            }
            RAM_BANK_NUMBER_START..=RAM_BANK_NUMBER_END => {
                let bank_mask = (self.ram_size.banks().saturating_sub(1));
                self.ram_bank = u16::from(byte) & (0x03 & bank_mask);

                // TODO: 1MB MBC1 ROM switching, see TODO above
            }
            _ => {}
        }
    }

    pub(super) fn read_ram(&self, address: u16) -> u8 {
        match self.ram_size {
            RamSize::Unset => BYTE_INVALID_READ,
            RamSize::Extended(size, banks) => {
                self.ram[usize::from(address)
                    + (usize::from(self.ram_bank) * usize::from(RAM_BANK_SIZE))]
            }
        }
    }

    pub(super) fn write_ram(&mut self, address: u16, byte: u8) {
        match self.ram_size {
            RamSize::Unset => return,
            RamSize::Extended(size, banks) => {
                self.ram[usize::from(address)
                    + (usize::from(self.ram_bank) * usize::from(RAM_BANK_SIZE))] = byte
            }
        }
    }
}

#[derive(Clone)]
pub struct Addressible<const S: usize> {
    memory: [u8; S],
}

impl<const S: usize> Default for Addressible<S> {
    fn default() -> Self {
        Self { memory: [0; S] }
    }
}

impl<const S: usize> Addressible<S> {
    pub(super) fn read(&self, address: u16) -> u8 {
        let address: usize = address.into();
        self.memory[address]
    }

    pub(super) fn write(&mut self, address: u16, byte: u8) {
        let address: usize = address.into();
        self.memory[address] = byte;
    }
}
