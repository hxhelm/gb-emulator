use super::bus::Bus;

impl Bus {
    pub fn get_lcd_control(&self) -> u8 {
        self.io.lcd_control
    }

    pub fn set_lcd_control(&mut self, value: u8) {
        self.io.lcd_control = value;
    }

    pub fn get_lcd_stat(&self) -> u8 {
        self.io.lcd_stat
    }

    pub fn set_lcd_stat(&mut self, value: u8) {
        self.io.lcd_stat = value;
    }

    pub fn get_lcd_y(&self) -> u8 {
        self.io.lcd_y
    }

    pub fn set_lcd_y(&mut self, value: u8) {
        self.io.lcd_y = value % 154; // there are 154 scanlines
    }

    pub fn get_lcd_y_compare(&self) -> u8 {
        self.io.lcd_y_compare
    }

    pub fn get_window_y(&self) -> u8 {
        self.io.window_y
    }

    pub fn get_window_x(&self) -> u8 {
        self.io.window_x.wrapping_sub(7)
    }

    pub fn get_scroll_y(&self) -> u8 {
        self.io.scroll_y
    }

    pub fn get_scroll_x(&self) -> u8 {
        self.io.scroll_x
    }
}
