#![allow(dead_code)]
use crate::memory::bus::{get_bit_status, set_bit_status, Bus};

use super::PPUMode;

/// LCDC register address
pub const LCD_CONTROL: u16 = 0xFF40;
const LCDC_BIT_LCD_ENABLE: u8 = 7;
const LCDC_BIT_WINDOW_TILE_MAP: u8 = 6;
const LCDC_BIT_WINDOW_ENABLE: u8 = 5;
const LCDC_BIT_BG_WINDOW_TILE_DATA_AREA: u8 = 4;
const LCDC_BIT_BG_TILE_MAP: u8 = 3;
const LCDC_BIT_OBJ_SIZE: u8 = 2;
const LCDC_BIT_OBJ_ENABLE: u8 = 1;
const LCDC_BIT_BG_WINDOW_ENABLE: u8 = 0;

impl Bus {
    fn get_lcdc_bit(&self, position: u8) -> bool {
        get_bit_status(self.read_byte(LCD_CONTROL), position)
    }

    fn set_lcdc_bit(&mut self, position: u8, status: bool) {
        let current = self.read_byte(LCD_CONTROL);
        let new = set_bit_status(current, position, status);
        self.write_byte(LCD_CONTROL, new);
    }

    /// LCDC.7: Returns whether LCD & PPU are enabled
    pub(crate) fn lcd_enabled(&self) -> bool {
        self.get_lcdc_bit(LCDC_BIT_LCD_ENABLE)
    }

    /// LCDC.5: Returns whether window is enabled
    pub(crate) fn window_enabled(&self) -> bool {
        self.get_lcdc_bit(LCDC_BIT_WINDOW_ENABLE)
    }
}

const VRAM_TILE_MAP_AREA_0_START: u16 = 0x9800;
const VRAM_TILE_MAP_AREA_0_END: u16 = 0x9BFF;
const VRAM_TILE_MAP_AREA_1_START: u16 = 0x9C00;
const VRAM_TILE_MAP_AREA_1_END: u16 = 0x9FFF;

pub(crate) enum TileMapArea {
    Area0,
    Area1,
}

impl TileMapArea {
    pub fn start(&self) -> u16 {
        match *self {
            Self::Area0 => 0x9800,
            Self::Area1 => 0x9C00,
        }
    }

    pub fn end(&self) -> u16 {
        match *self {
            Self::Area0 => 0x9BFF,
            Self::Area1 => 0x9FFF,
        }
    }
}

impl Bus {
    /// LCDC.6: Returns window tile map area
    pub(crate) fn get_window_tile_map(&self) -> TileMapArea {
        if self.get_lcdc_bit(LCDC_BIT_WINDOW_TILE_MAP) {
            TileMapArea::Area1
        } else {
            TileMapArea::Area0
        }
    }

    /// LCDC.3: Returns background tile map area
    pub(crate) fn get_bg_tile_map(&self) -> TileMapArea {
        if self.get_lcdc_bit(LCDC_BIT_BG_TILE_MAP) {
            TileMapArea::Area1
        } else {
            TileMapArea::Area0
        }
    }
}

pub(crate) enum TileDataArea {
    /// 0x8000 method: unsigned addressing, tiles 0-127 -> block 0, tiles 128-255 -> block 1
    Method8000,
    /// 0x8800 method: signed addressing, tiles 0-127 -> block 2, tiles 128-255 -> block 1
    Method8800,
}

impl TileDataArea {
    pub(crate) fn get_tile_address(&self, tile_number: u8) -> u16 {
        match self {
            TileDataArea::Method8000 => 0x8000u16.wrapping_add((tile_number as u16) * 16),
            TileDataArea::Method8800 => {
                let signed_tile = tile_number as i8; // Interpret as signed
                0x9000u16.wrapping_add_signed((signed_tile as i16) * 16)
            }
        }
    }
}

impl Bus {
    /// LCDC.4: Returns window & background tile data area address mode
    pub(crate) fn get_bg_window_tile_data_area(&self) -> TileDataArea {
        if self.get_lcdc_bit(LCDC_BIT_BG_WINDOW_TILE_DATA_AREA) {
            TileDataArea::Method8000
        } else {
            TileDataArea::Method8800
        }
    }

    /// LCDC.4: Set window & background tile data area address mode
    pub(crate) fn set_bg_window_tile_data_area(&mut self, status: bool) {
        self.set_lcdc_bit(LCDC_BIT_BG_WINDOW_TILE_DATA_AREA, status)
    }
}

impl Bus {
    /// LCDC.2: Returns the size of all objects (1 tile or 2 stacked vertically)
    pub(crate) fn get_obj_size(&self) -> bool {
        self.get_lcdc_bit(LCDC_BIT_OBJ_SIZE)
    }

    /// LCDC.2: Returns whether objects are displayed or not
    pub(crate) fn objects_enabled(&self) -> bool {
        self.get_lcdc_bit(LCDC_BIT_OBJ_ENABLE)
    }

    pub(crate) fn bg_window_enabled(&self) -> bool {
        self.get_lcdc_bit(LCDC_BIT_BG_WINDOW_ENABLE)
    }
}

/// STAT register address
pub const LCD_STAT: u16 = 0xFF41;
const LCD_STAT_BIT_LYC: u8 = 2;
const LCD_STAT_BIT_MODE: u8 = 0;
/// LY register address
pub const LCD_Y: u16 = 0xFF44;
/// LYC register address
pub const LCD_Y_COMPARE: u16 = 0xFF45;

pub(super) struct StatCondition {
    pub(super) lyc: bool,
    pub(super) mode2: bool,
    pub(super) mode1: bool,
    pub(super) mode0: bool,
}

impl Bus {
    pub(super) fn lcd_status_condition(&self) -> StatCondition {
        let stat = self.read_byte(LCD_STAT) >> 3;

        StatCondition {
            lyc: stat & 0b1000 != 0,
            mode2: stat & 0b100 != 0,
            mode1: stat & 0b10 != 0,
            mode0: stat & 0b1 != 0,
        }
    }

    pub(super) fn lcd_status_set_lyc(&mut self, condition: bool) {
        let value = set_bit_status(self.read_byte(LCD_STAT), LCD_STAT_BIT_LYC, condition);
        self.write_byte(LCD_STAT, value);
    }

    pub(super) fn lcd_status_set_mode(&mut self, mode: PPUMode) {
        let value = match mode {
            PPUMode::OBJSearch => 0b10,
            PPUMode::SendPixels => 0b11,
            PPUMode::HorizontalBlank => 0,
            PPUMode::VerticalBlank => 0b1,
        };

        let current = self.read_byte(LCD_STAT);
        self.write_byte(LCD_STAT, current & (0xFC | value));
    }

    pub(crate) fn lcd_update_line(&mut self) {
        let ly = self.read_byte(LCD_Y);
        self.write_byte(LCD_Y, (ly + 1) % 154);
        self.update_stat_lyc();
    }

    pub(crate) fn lcd_current_line(&self) -> u8 {
        self.read_byte(LCD_Y)
    }

    pub(super) fn lcd_y_compare(&self) -> bool {
        self.read_byte(LCD_Y) == self.read_byte(LCD_Y_COMPARE)
    }

    pub fn update_stat_lyc(&mut self) {
        self.lcd_status_set_lyc(self.lcd_y_compare());

        if self.lcd_status_condition().lyc && self.lcd_y_compare() {
            // eprintln!(
            //     "Requested STAT interrupt. Scanline: {}",
            //     self.lcd_current_line()
            // );
            self.request_stat_interrupt();
        }
    }
}

/// WY register address
pub const WINDOW_Y: u16 = 0xFF4A;
/// WX register address
pub const WINDOW_X: u16 = 0xFF4B;
/// SCY register address
pub const SCROLL_Y: u16 = 0xFF42;
/// SCX register address
pub const SCROLL_X: u16 = 0xFF43;

impl Bus {
    pub(crate) fn get_window_y(&self) -> u8 {
        self.read_byte(WINDOW_Y)
    }

    pub(crate) fn get_window_x(&self) -> u8 {
        self.read_byte(WINDOW_X).wrapping_sub(7)
    }

    pub(crate) fn get_scroll_x(&self) -> u8 {
        self.read_byte(SCROLL_X)
    }

    pub(crate) fn get_scroll_y(&self) -> u8 {
        self.read_byte(SCROLL_Y)
    }
}
