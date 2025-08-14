#![allow(dead_code)]
use crate::memory::bus::{get_bit_status, set_bit_status, Bus};

use super::PPUMode;

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
        get_bit_status(self.get_lcd_control(), position)
    }

    fn set_lcdc_bit(&mut self, position: u8, status: bool) {
        let current = self.get_lcd_control();
        let new = set_bit_status(current, position, status);
        self.set_lcd_control(new)
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

    pub(crate) fn get_object_tile_data_area(&self) -> TileDataArea {
        TileDataArea::Method8000
    }
}

pub const OBJECT_SIZE: u8 = 16;

impl Bus {
    /// LCDC.2: Returns the size of all objects (1 tile or 2 stacked vertically)
    pub(crate) fn get_obj_size(&self) -> u8 {
        if self.get_lcdc_bit(LCDC_BIT_OBJ_SIZE) {
            OBJECT_SIZE * 2
        } else {
            OBJECT_SIZE
        }
    }

    /// LCDC.1: Returns whether objects are displayed
    pub(crate) fn objects_enabled(&self) -> bool {
        self.get_lcdc_bit(LCDC_BIT_OBJ_ENABLE)
    }

    /// LCDC.0: Return whether the background & window are displayed
    pub(crate) fn bg_window_enabled(&self) -> bool {
        self.get_lcdc_bit(LCDC_BIT_BG_WINDOW_ENABLE)
    }
}

const LCD_STAT_BIT_LYC: u8 = 2;
const LCD_STAT_BIT_MODE: u8 = 0;

pub(super) struct StatCondition {
    pub(super) lyc: bool,
    pub(super) mode2: bool,
    pub(super) mode1: bool,
    pub(super) mode0: bool,
}

impl Bus {
    pub(super) fn lcd_status_condition(&self) -> StatCondition {
        let stat = self.get_lcd_stat() >> 3;

        StatCondition {
            lyc: stat & 0b1000 != 0,
            mode2: stat & 0b100 != 0,
            mode1: stat & 0b10 != 0,
            mode0: stat & 0b1 != 0,
        }
    }

    pub(super) fn lcd_status_set_lyc(&mut self, condition: bool) {
        let value = set_bit_status(self.get_lcd_stat(), LCD_STAT_BIT_LYC, condition);
        self.set_lcd_stat(value);
    }

    pub(super) fn lcd_status_set_mode(&mut self, mode: PPUMode) {
        let value = match mode {
            PPUMode::OBJSearch => 0b10,
            PPUMode::SendPixels => 0b11,
            PPUMode::HorizontalBlank => 0,
            PPUMode::VerticalBlank => 0b1,
        };

        let current = self.get_lcd_stat();
        self.set_lcd_stat(current & (0xFC | value));
    }

    pub(crate) fn update_line(&mut self) {
        self.set_lcd_y(self.get_lcd_y() + 1);
        self.update_stat_lyc();
    }

    pub(crate) fn current_line(&self) -> u8 {
        self.get_lcd_y()
    }

    pub(super) fn lcd_y_compare(&self) -> bool {
        self.get_lcd_y() == self.get_lcd_y_compare()
    }

    pub fn update_stat_lyc(&mut self) {
        self.lcd_status_set_lyc(self.lcd_y_compare());

        if self.lcd_status_condition().lyc && self.lcd_y_compare() {
            self.request_stat_interrupt();
        }
    }
}
