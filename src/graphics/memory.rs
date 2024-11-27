use crate::memory::bus::Bus;

const _LCD_CONTROL: u16 = 0xFF40;
const LCD_Y_COORDINATE: u16 = 0xFF44;
const _LCD_Y_COMPARE: u16 = 0xFF45;
const _LCD_STAT: u16 = 0xFF41;

impl Bus {
    pub(crate) fn lcd_update_line(&mut self) {
        let ly = self.read_byte(LCD_Y_COORDINATE);
        self.write_byte(LCD_Y_COORDINATE, (ly + 1) % 154)
    }

    pub(crate) fn lcd_current_line(&self) -> u8 {
        self.read_byte(LCD_Y_COORDINATE)
    }
}
