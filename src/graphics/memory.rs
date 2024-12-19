#![allow(dead_code)]
use crate::memory::bus::Bus;

const LCD_CONTROL: u16 = 0xFF40;
const LCDC_BIT_LCD_ENABLE: u8 = 7;
const LCDC_BIT_WINDOW_TILE_MAP: u8 = 6;
const LCDC_BIT_WINDOW_ENABLE: u8 = 5;
const LCDC_BIT_BG_WINDOW_TILE_DATA_AREA: u8 = 4;
const LCDC_BIT_BG_TILE_MAP: u8 = 3;
const LCDC_BIT_OBJ_SIZE: u8 = 2;
const LCDC_BIT_OBJ_ENABLE: u8 = 1;
const LCDC_BIT_BG_WINDOW_ENABLE: u8 = 0;

fn get_bit_status(byte: u8, position: u8) -> bool {
    byte & (1 << position) != 0
}

fn set_bit_status(byte: u8, position: u8, status: bool) -> u8 {
    if status {
        byte | 1 << position
    } else {
        byte & !(1 << position)
    }
}

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
    pub(crate) fn get_lcd_enable(&self) -> bool {
        self.get_lcdc_bit(LCDC_BIT_LCD_ENABLE)
    }

    /// LCDC.7: Set whether LCD & PPU are enabled
    pub(crate) fn set_lcd_enable(&mut self, status: bool) {
        self.set_lcdc_bit(LCDC_BIT_LCD_ENABLE, status)
    }

    /// LCDC.5: Returns whether window is enabled
    pub(crate) fn get_window_enable(&self) -> bool {
        self.get_lcdc_bit(LCDC_BIT_WINDOW_ENABLE)
    }

    /// LCDC.5: Set whether window is enabled
    pub(crate) fn set_window_enable(&mut self, status: bool) {
        self.set_lcdc_bit(LCDC_BIT_WINDOW_ENABLE, status)
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
    /// LDCD.6: Returns window tile map area
    pub(crate) fn get_window_tile_map(&self) -> TileMapArea {
        if self.get_lcdc_bit(LCDC_BIT_WINDOW_TILE_MAP) {
            TileMapArea::Area1
        } else {
            TileMapArea::Area0
        }
    }

    /// LDCD.6: Set window tile map area
    pub(crate) fn set_window_tile_map(&mut self, status: bool) {
        self.set_lcdc_bit(LCDC_BIT_WINDOW_TILE_MAP, status)
    }

    /// LCDC.3: Returns background tile map area
    pub(crate) fn get_bg_tile_map(&self) -> TileMapArea {
        if self.get_lcdc_bit(LCDC_BIT_BG_TILE_MAP) {
            TileMapArea::Area1
        } else {
            TileMapArea::Area0
        }
    }

    /// LCDC.3: Set background tile map area
    pub(crate) fn set_bg_tile_map(&mut self, status: bool) {
        self.set_lcdc_bit(LCDC_BIT_BG_TILE_MAP, status)
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
            TileDataArea::Method8000 => {
                // Tiles 0–255, starting at 0x8000
                0x8000 + (tile_number as u16) * 16
            }
            TileDataArea::Method8800 => {
                // Tiles -128 to 127 (signed), starting at 0x8800
                let signed_tile = tile_number as i8; // Interpret as signed
                0x9000u16.wrapping_add_signed((signed_tile as i16) * 16) as u16
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

    /// LCDC.2: Set the size of all objects (1 tile or 2 stacked vertically)
    pub(crate) fn set_obj_size(&mut self, status: bool) {
        self.set_lcdc_bit(LCDC_BIT_OBJ_SIZE, status)
    }

    /// LCDC.2: Returns whether objects are displayed or not
    pub(crate) fn get_obj_enable(&self) -> bool {
        self.get_lcdc_bit(LCDC_BIT_OBJ_ENABLE)
    }

    /// LCDC.2: Set whether objects are displayed or not
    pub(crate) fn set_obj_enable(&mut self, status: bool) {
        self.set_lcdc_bit(LCDC_BIT_OBJ_ENABLE, status)
    }

    pub(crate) fn get_bg_window_enable(&self) -> bool {
        self.get_lcdc_bit(LCDC_BIT_BG_WINDOW_ENABLE)
    }

    pub(crate) fn set_bg_window_enable(&mut self, status: bool) {
        self.set_lcdc_bit(LCDC_BIT_BG_WINDOW_ENABLE, status)
    }
}

const LCD_Y: u16 = 0xFF44;
const _LCD_Y_COMPARE: u16 = 0xFF45;
const _LCD_STAT: u16 = 0xFF41;
const WINDOW_Y: u16 = 0xFF4A;
const WINDOW_X: u16 = 0xFF4A;
const SCROLL_X: u16 = 0xFF43;
const SCROLL_Y: u16 = 0xFF42;

impl Bus {
    pub(crate) fn lcd_update_line(&mut self) {
        let ly = self.read_byte(LCD_Y);
        self.write_byte(LCD_Y, (ly + 1) % 154)
    }

    pub(crate) fn lcd_current_line(&self) -> u8 {
        self.read_byte(LCD_Y)
    }

    pub(crate) fn get_window_y(&self) -> u8 {
        self.read_byte(WINDOW_Y)
    }

    pub(crate) fn get_window_x(&self) -> u8 {
        self.read_byte(WINDOW_X).saturating_sub(7)
    }

    pub(crate) fn get_scroll_x(&self) -> u8 {
        self.read_byte(SCROLL_X)
    }

    pub(crate) fn get_scroll_y(&self) -> u8 {
        self.read_byte(SCROLL_Y)
    }
}
