#![allow(unused)]
use std::path::Iter;

use super::mem::{Addressible, Memory};
use crate::graphics::PPUMode;

pub const BUS_SIZE: usize = 0xFFFF + 1;
const ROM_BANK_0_START: u16 = 0x0000;
const ROM_BANK_0_END: u16 = 0x3FFF;
const ROM_BANK_1_START: u16 = 0x4000;
const ROM_BANK_1_END: u16 = 0x7FFF;
const VRAM_START: u16 = 0x8000;
const VRAM_END: u16 = 0x9FFF;
const VRAM_SIZE: usize = (VRAM_END - VRAM_START + 1) as usize;
const EXTERNAL_RAM_START: u16 = 0xA000;
const EXTERNAL_RAM_END: u16 = 0xBFFF;
const EXTERNAL_RAM_SIZE: usize = (EXTERNAL_RAM_END - EXTERNAL_RAM_START + 1) as usize;
const WRAM_START: u16 = 0xC000;
const WRAM_END: u16 = 0xDFFF;
const WRAM_SIZE: usize = (WRAM_END - WRAM_START + 1) as usize;
const OAM_START: u16 = 0xFE00;
const OAM_END: u16 = 0xFEFF;
const OAM_SIZE: usize = (OAM_END - OAM_START + 1) as usize;
const HRAM_START: u16 = 0xFF80;
const HRAM_END: u16 = 0xFFFE;
const HRAM_SIZE: usize = (HRAM_END - HRAM_START + 1) as usize;
const BOOT_ROM_LENGTH: u16 = 0x0100;

pub(super) const BYTE_INVALID_READ: u8 = 0xFF;

#[derive(Clone)]
pub struct Bus {
    cartridge: Memory,
    vram: Addressible<VRAM_SIZE>,
    wram: Addressible<WRAM_SIZE>,
    // TODO: OAM DMA transfer https://gbdev.io/pandocs/OAM_DMA_Transfer.html#oam-dma-transfer
    pub oam: Addressible<OAM_SIZE>,
    pub(super) io: IORegisters,
    hram: Addressible<HRAM_SIZE>,
    pub ppu_mode: PPUMode,
    /// boot rom is saved in separate space, as it is unmapped after boot and saved inside the CPU
    boot_rom: [u8; BOOT_ROM_LENGTH as usize],
    pub boot_rom_disabled: bool,
}

pub const CARTRIDGE_TYPE: u16 = 0x0147;
pub const CARTRIDGE_ROM_SIZE: u16 = 0x0148;
pub const CARTRIDGE_RAM_SIZE: u16 = 0x0149;

/// JOYPAD input register address
pub const JOYP: u16 = 0xFF00;
/// SB register address
pub const SERIAL_TRANSFER_DATA: u16 = 0xFF01;
/// SC register address
pub const SERIAL_TRANSFER_CONTROL: u16 = 0xFF02;
/// DIV register address
pub const TIMER_DIVIDER: u16 = 0xFF04;
/// TIMA register address
pub const TIMER_COUNTER: u16 = 0xFF05;
/// TMA register address
pub const TIMER_MODULO: u16 = 0xFF06;
/// TAC register address
pub const TIMER_CONTROL: u16 = 0xFF07;
/// IF register address
pub const INTERRUPT_REQUESTS: u16 = 0xFF0F;
/// IE register address
pub const INTERRUPT_ENABLE: u16 = 0xFFFF;
/// LCDC register address
pub const LCD_CONTROL: u16 = 0xFF40;
/// STAT register address
pub const LCD_STAT: u16 = 0xFF41;
/// LY register address
pub const LCD_Y: u16 = 0xFF44;
/// LYC register address
pub const LCD_Y_COMPARE: u16 = 0xFF45;
/// WY register address
pub const WINDOW_Y: u16 = 0xFF4A;
/// WX register address
pub const WINDOW_X: u16 = 0xFF4B;
/// SCY register address
pub const SCROLL_Y: u16 = 0xFF42;
/// SCX register address
pub const SCROLL_X: u16 = 0xFF43;
const BOOT_DISABLE: u16 = 0xFF50;

#[derive(Default, Clone, Copy)]
pub(super) struct IORegisters {
    pub(super) joypad: u8,
    pub(super) serial_data: u8,
    pub(super) serial_control: u8,
    pub(super) timer_divider: u8,
    pub(super) timer_counter: u8,
    pub(super) timer_modulo: u8,
    pub(super) timer_control: u8,
    pub(super) interrupt_requests: u8,
    pub(super) interrupt_enable: u8,
    // TODO: audio
    // TODO: wave pattern
    pub(super) boot_rom_disable: u8,
    pub(super) lcd_control: u8,
    pub(super) lcd_stat: u8,
    pub(super) lcd_y: u8,
    pub(super) lcd_y_compare: u8,
    pub(super) window_y: u8,
    pub(super) window_x: u8,
    pub(super) scroll_y: u8,
    pub(super) scroll_x: u8,
}

impl Bus {
    pub fn from_cartridge(cartridge_contents: &[u8]) -> Self {
        Self {
            cartridge: Memory::init(cartridge_contents),
            vram: Addressible::default(),
            wram: Addressible::default(),
            oam: Addressible::default(),
            io: IORegisters::default(),
            hram: Addressible::default(),
            ppu_mode: PPUMode::default(),
            boot_rom: [0; BOOT_ROM_LENGTH as usize],
            boot_rom_disabled: false,
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            ROM_BANK_0_START..=ROM_BANK_1_END => {
                if self.boot_rom_disabled {
                    self.cartridge.read_rom(address)
                } else {
                    match address {
                        0..BOOT_ROM_LENGTH => self.boot_rom[address as usize],
                        _ => self.cartridge.read_rom(address),
                    }
                }
            }
            VRAM_START..=VRAM_END => {
                if matches!(self.ppu_mode, PPUMode::SendPixels) {
                    BYTE_INVALID_READ
                } else {
                    self.vram.read(address - VRAM_START)
                }
            }
            EXTERNAL_RAM_START..=EXTERNAL_RAM_END => {
                self.cartridge.read_ram(address - EXTERNAL_RAM_START)
            }
            WRAM_START..=WRAM_END => self.wram.read(address - WRAM_START),
            0xE000..=0xFDFF => {
                unreachable!("Tried to access prohibited bus address 0x{address:04X}")
            }
            OAM_START..=OAM_END => {
                if matches!(self.ppu_mode, PPUMode::OBJSearch | PPUMode::SendPixels) {
                    BYTE_INVALID_READ
                } else {
                    self.oam.read(address - OAM_START)
                }
            }
            HRAM_START..=HRAM_END => self.hram.read(address - HRAM_START),
            JOYP => self.io.joypad,
            SERIAL_TRANSFER_DATA => self.io.serial_data,
            SERIAL_TRANSFER_CONTROL => self.io.serial_control,
            TIMER_DIVIDER => self.io.timer_divider,
            TIMER_COUNTER => self.io.timer_counter,
            TIMER_MODULO => self.io.timer_modulo,
            TIMER_CONTROL => self.io.timer_control,
            INTERRUPT_REQUESTS => self.io.interrupt_requests,
            INTERRUPT_ENABLE => self.io.interrupt_enable,
            LCD_CONTROL => self.io.lcd_control,
            LCD_STAT => self.io.lcd_stat,
            LCD_Y => self.io.lcd_y,
            LCD_Y_COMPARE => self.io.lcd_y_compare,
            WINDOW_Y => self.io.window_y,
            WINDOW_X => self.io.window_x,
            SCROLL_Y => self.io.scroll_y,
            SCROLL_X => self.io.scroll_x,
            BOOT_DISABLE => self.io.boot_rom_disable,
            // _ => unimplemented!("Tried to read from unmapped bus address 0x{address:04X}"),
            _ => BYTE_INVALID_READ,
        }
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            // we don't check for the boot rom area here because the boot rom does not write in its
            // own address space
            ROM_BANK_0_START..=ROM_BANK_1_END => self.cartridge.write_rom(address, byte),
            VRAM_START..=VRAM_END => {
                if !matches!(self.ppu_mode, PPUMode::SendPixels) {
                    self.vram.write(address - VRAM_START, byte)
                }
            }
            EXTERNAL_RAM_START..=EXTERNAL_RAM_END => {
                self.cartridge.write_ram(address - EXTERNAL_RAM_START, byte)
            }
            WRAM_START..=WRAM_END => self.wram.write(address - WRAM_START, byte),
            0xE000..=0xFDFF => {
                unreachable!("Tried to write prohibited bus address 0x{address:04X}")
            }
            OAM_START..=OAM_END => {
                if !matches!(self.ppu_mode, PPUMode::OBJSearch | PPUMode::SendPixels) {
                    self.oam.write(address - OAM_START, byte)
                }
            }
            HRAM_START..=HRAM_END => self.hram.write(address - HRAM_START, byte),
            JOYP => self.io.joypad = byte,
            SERIAL_TRANSFER_DATA => self.io.serial_data = byte,
            SERIAL_TRANSFER_CONTROL => self.io.serial_control = byte,
            TIMER_DIVIDER => self.io.timer_divider = byte,
            TIMER_COUNTER => self.io.timer_counter = byte,
            TIMER_MODULO => self.io.timer_modulo = byte,
            TIMER_CONTROL => self.io.timer_control = byte,
            INTERRUPT_REQUESTS => self.io.interrupt_requests = byte,
            INTERRUPT_ENABLE => self.io.interrupt_enable = byte,
            LCD_CONTROL => self.set_lcd_control(byte),
            LCD_STAT => self.set_lcd_stat(byte),
            LCD_Y => self.set_lcd_y(byte),
            LCD_Y_COMPARE => {
                self.io.lcd_y_compare = byte;
                self.update_stat_lyc();
            }
            WINDOW_Y => self.io.window_y = byte,
            WINDOW_X => self.io.window_x = byte,
            SCROLL_Y => self.io.scroll_y = byte,
            SCROLL_X => self.io.scroll_x = byte,
            BOOT_DISABLE => {
                self.boot_rom_disabled = byte != 0;
                self.io.boot_rom_disable = byte
            }
            _ => {}
        }
    }

    /// Ignores memory locations blocking by the PPU Mode, mainly used for the PPU itself
    pub fn ppu_read(&self, address: u16) -> u8 {
        match address {
            VRAM_START..=VRAM_END => self.vram.read(address - VRAM_START),
            OAM_START..=OAM_END => self.oam.read(address - OAM_START),
            _ => unreachable!("Invalid PPU read of address 0x{address:04X}"),
        }
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

    pub fn read_word(&self, address: u16) -> u16 {
        let lsb = self.read_byte(address);
        let msb = self.read_byte(address + 1);

        u16::from_le_bytes([lsb, msb])
    }

    pub fn update_ppu_mode(&mut self, mode: PPUMode) {
        self.ppu_mode = mode;
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

    pub fn read_debug(&self, address: u16) -> u8 {
        match address {
            ROM_BANK_0_START..=ROM_BANK_1_END => {
                if self.boot_rom_disabled {
                    self.cartridge.read_rom(address)
                } else {
                    match address {
                        0..BOOT_ROM_LENGTH => self.boot_rom[address as usize],
                        _ => self.cartridge.read_rom(address),
                    }
                }
            }
            VRAM_START..=VRAM_END => self.vram.read(address - VRAM_START),
            EXTERNAL_RAM_START..=EXTERNAL_RAM_END => {
                self.cartridge.read_ram(address - EXTERNAL_RAM_START)
            }
            WRAM_START..=WRAM_END => self.wram.read(address - WRAM_START),
            OAM_START..=OAM_END => self.oam.read(address - OAM_START),
            HRAM_START..=HRAM_END => self.hram.read(address - HRAM_START),
            JOYP => self.io.joypad,
            SERIAL_TRANSFER_DATA => self.io.serial_data,
            SERIAL_TRANSFER_CONTROL => self.io.serial_control,
            TIMER_DIVIDER => self.io.timer_divider,
            TIMER_COUNTER => self.io.timer_counter,
            TIMER_MODULO => self.io.timer_modulo,
            TIMER_CONTROL => self.io.timer_control,
            INTERRUPT_REQUESTS => self.io.interrupt_requests,
            INTERRUPT_ENABLE => self.io.interrupt_enable,
            LCD_CONTROL => self.io.lcd_control,
            LCD_STAT => self.io.lcd_stat,
            LCD_Y => self.io.lcd_y,
            LCD_Y_COMPARE => self.io.lcd_y_compare,
            WINDOW_Y => self.io.window_y,
            WINDOW_X => self.io.window_x,
            SCROLL_Y => self.io.scroll_y,
            SCROLL_X => self.io.scroll_x,
            BOOT_DISABLE => self.io.boot_rom_disable,
            _ => 0xFF,
        }
    }

    pub fn read_range_debug(&self, address: u16, length: u16) -> Vec<u8> {
        (address..=(address.saturating_add(length)))
            .map(|address| self.read_debug(address))
            .collect()
    }
}

pub const fn get_bit_status(byte: u8, position: u8) -> bool {
    byte & (1 << position) != 0
}

pub const fn set_bit_status(byte: u8, position: u8, status: bool) -> u8 {
    if status {
        byte | 1 << position
    } else {
        byte & !(1 << position)
    }
}
