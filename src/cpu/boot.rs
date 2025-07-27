use crate::cpu::CPU;
use crate::memory::bus::{
    INTERRUPT_ENABLE, INTERRUPT_REQUESTS, JOYP, LCD_Y_COMPARE, SCROLL_X, SCROLL_Y,
    SERIAL_TRANSFER_CONTROL, SERIAL_TRANSFER_DATA, TIMER_CONTROL, TIMER_COUNTER, TIMER_MODULO,
    WINDOW_X, WINDOW_Y,
};

impl CPU {
    pub(super) fn load_boot_rom(&mut self, boot_rom: &[u8]) {
        self.bus.write_boot_rom(boot_rom);

        // Stub JOYPAD Status to 0xCF until proper implementation
        self.bus.write_byte(JOYP, 0xCF);
    }

    /// Set state according to https://gbdev.io/pandocs/Power_Up_Sequence.html
    pub(super) fn init_boot_handoff(&mut self) {
        self.bus.boot_rom_disabled = true;

        self.registers.a = 0x01;
        self.registers.b = 0x00;
        self.registers.c = 0x13;
        self.registers.d = 0x00;
        self.registers.e = 0xD8;
        self.registers.h = 0x01;
        self.registers.l = 0x4D;
        self.registers.pc = 0x0100;
        self.registers.sp = 0xFFFE;

        let header_checksum = self.calculate_cartridge_header_checksum();
        self.registers.f.zero = true;
        self.registers.f.negative = false;
        self.registers.f.half_carry = header_checksum != 0;
        self.registers.f.carry = header_checksum != 0;

        self.bus.write_byte(JOYP, 0xCF);
        self.bus.write_byte(SERIAL_TRANSFER_DATA, 0x00);
        self.bus.write_byte(SERIAL_TRANSFER_CONTROL, 0x7E);

        // DIV
        // self.bus.write_byte(0xFF04, 0xAB);

        self.bus.write_byte(TIMER_COUNTER, 0x00);
        self.bus.write_byte(TIMER_MODULO, 0x00);
        self.bus.write_byte(TIMER_CONTROL, 0xF8);
        self.bus.write_byte(INTERRUPT_REQUESTS, 0xE1);

        // NR
        // self.bus.write_byte(0xFF10, 0x80);
        // self.bus.write_byte(0xFF11, 0xBF);
        // self.bus.write_byte(0xFF12, 0xF3);
        // self.bus.write_byte(0xFF13, 0xFF);
        // self.bus.write_byte(0xFF14, 0xBF);
        // self.bus.write_byte(0xFF16, 0x3F);
        // self.bus.write_byte(0xFF17, 0x00);
        // self.bus.write_byte(0xFF18, 0xFF);
        // self.bus.write_byte(0xFF19, 0xBF);
        // self.bus.write_byte(0xFF1A, 0x7F);
        // self.bus.write_byte(0xFF1B, 0xFF);
        // self.bus.write_byte(0xFF1C, 0x9F);
        // self.bus.write_byte(0xFF1D, 0xFF);
        // self.bus.write_byte(0xFF1E, 0xBF);
        // self.bus.write_byte(0xFF20, 0xFF);
        // self.bus.write_byte(0xFF21, 0x00);
        // self.bus.write_byte(0xFF22, 0x00);
        // self.bus.write_byte(0xFF23, 0xBF);
        // self.bus.write_byte(0xFF24, 0x77);
        // self.bus.write_byte(0xFF25, 0xF3);
        // self.bus.write_byte(0xFF26, 0xF1);

        self.bus.set_lcd_control(0x91);
        self.bus.set_lcd_stat(0x85);
        self.bus.write_byte(SCROLL_Y, 0x00);
        self.bus.write_byte(SCROLL_X, 0x00);
        self.bus.set_lcd_y(0x00);
        self.bus.write_byte(LCD_Y_COMPARE, 0x00);

        // DMA
        // self.bus.write_byte(0xFF46, 0xFF);

        // BGP
        // self.bus.write_byte(0xFF47, 0xFC);
        // OBP0
        // OBP1

        self.bus.write_byte(WINDOW_Y, 0x00);
        self.bus.write_byte(WINDOW_X, 0x00);
        self.bus.write_byte(INTERRUPT_ENABLE, 0x00)
    }

    fn calculate_cartridge_header_checksum(&self) -> u8 {
        let mut checksum: u8 = 0;

        for address in 0x0134..=0x014C {
            checksum = checksum.wrapping_sub(self.bus.read_byte(address) + 1);
        }

        checksum
    }
}
