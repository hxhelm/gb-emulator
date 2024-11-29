#![allow(dead_code)]
use crate::memory::bus::Bus;

const LCD_CONTROL: u16 = 0xFF40;
const LCDC_BIT_LCD_ENABLE: u8 = 7;
const LCDC_BIT_WINDOW_TILE_MAP: u8 = 6;
const LCDC_BIT_WINDOW_ENABLE: u8 = 5;
const LCDC_BIT_BG_WINDOW_TILE_DATA: u8 = 4;
const LCDC_BIT_TILE_MAP: u8 = 3;
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

    pub(crate) fn get_lcd_enable(&self) -> bool {
        self.get_lcdc_bit(LCDC_BIT_LCD_ENABLE)
    }

    pub(crate) fn set_lcd_enable(&mut self, status: bool) {
        self.set_lcdc_bit(LCDC_BIT_LCD_ENABLE, status)
    }

    pub(crate) fn get_window_tile_map(&self) -> bool {
        self.get_lcdc_bit(LCDC_BIT_WINDOW_TILE_MAP)
    }

    pub(crate) fn set_window_tile_map(&mut self, status: bool) {
        self.set_lcdc_bit(LCDC_BIT_WINDOW_TILE_MAP, status)
    }

    pub(crate) fn get_window_enable(&self) -> bool {
        self.get_lcdc_bit(LCDC_BIT_WINDOW_ENABLE)
    }

    pub(crate) fn set_window_enable(&mut self, status: bool) {
        self.set_lcdc_bit(LCDC_BIT_WINDOW_ENABLE, status)
    }

    pub(crate) fn get_bg_window_tile_data(&self) -> bool {
        self.get_lcdc_bit(LCDC_BIT_BG_WINDOW_TILE_DATA)
    }

    pub(crate) fn set_bg_window_tile_data(&mut self, status: bool) {
        self.set_lcdc_bit(LCDC_BIT_BG_WINDOW_TILE_DATA, status)
    }

    pub(crate) fn get_tile_map(&self) -> bool {
        self.get_lcdc_bit(LCDC_BIT_TILE_MAP)
    }

    pub(crate) fn set_tile_map(&mut self, status: bool) {
        self.set_lcdc_bit(LCDC_BIT_TILE_MAP, status)
    }

    pub(crate) fn get_obj_size(&self) -> bool {
        self.get_lcdc_bit(LCDC_BIT_OBJ_SIZE)
    }

    pub(crate) fn set_obj_size(&mut self, status: bool) {
        self.set_lcdc_bit(LCDC_BIT_OBJ_SIZE, status)
    }

    pub(crate) fn get_obj_enable(&self) -> bool {
        self.get_lcdc_bit(LCDC_BIT_OBJ_ENABLE)
    }

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

impl Bus {
    pub(crate) fn lcd_update_line(&mut self) {
        let ly = self.read_byte(LCD_Y);
        self.write_byte(LCD_Y, (ly + 1) % 154)
    }

    pub(crate) fn lcd_current_line(&self) -> u8 {
        self.read_byte(LCD_Y)
    }

    pub(crate) fn window_y(&self) -> u8 {
        self.read_byte(WINDOW_Y)
    }

    pub(crate) fn window_x(&self) -> u8 {
        self.read_byte(WINDOW_X).saturating_sub(7)
    }
}
