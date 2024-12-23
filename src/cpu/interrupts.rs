#![allow(unused)]
use crate::cpu::{InterruptState, CPU};
use crate::memory::bus::{get_bit_status, set_bit_status, Bus};

const INTERRUPT_ENABLE: u16 = 0xFFFF;
const INTERRUPT_FLAG: u16 = 0xFF0F;
const INTERRUPT_VBLANK_BIT: u8 = 0b1;
const INTERRUPT_STAT_BIT: u8 = 0b10;
const INTERRUPT_TIMER_BIT: u8 = 0b100;
const INTERRUPT_SERIAL_BIT: u8 = 0b1000;
const INTERRUPT_JOYPAD_BIT: u8 = 0b10000;

#[derive(Clone, Copy)]
enum InterruptSource {
    VBLANK = 0x40,
    STAT = 0x48,
    TIMER = 0x50,
    SERIAL = 0x58,
    JOYPAD = 0x60,
}

impl Bus {
    pub fn request_vblank_interrupt(&mut self) {
        self.enable_interrupt_request(INTERRUPT_VBLANK_BIT)
    }

    pub fn request_stat_interrupt(&mut self) {
        self.enable_interrupt_request(INTERRUPT_STAT_BIT)
    }

    pub fn request_timer_interrupt(&mut self) {
        self.enable_interrupt_request(INTERRUPT_TIMER_BIT)
    }

    pub fn request_serial_interrupt(&mut self) {
        self.enable_interrupt_request(INTERRUPT_SERIAL_BIT)
    }

    pub fn request_joypad_interrupt(&mut self) {
        self.enable_interrupt_request(INTERRUPT_JOYPAD_BIT)
    }

    fn get_interrupt_enabled(&self) -> u8 {
        self.read_byte(INTERRUPT_ENABLE)
    }

    fn get_interrupt_flags(&self) -> u8 {
        self.read_byte(INTERRUPT_FLAG)
    }

    fn clear_interrupt_source(&mut self, source: &InterruptSource) {
        let bit = match source {
            InterruptSource::VBLANK => INTERRUPT_VBLANK_BIT,
            InterruptSource::STAT => INTERRUPT_STAT_BIT,
            InterruptSource::TIMER => INTERRUPT_TIMER_BIT,
            InterruptSource::SERIAL => INTERRUPT_SERIAL_BIT,
            InterruptSource::JOYPAD => INTERRUPT_JOYPAD_BIT,
        };

        self.clear_interrupt_request(bit)
    }

    fn clear_interrupt_request(&mut self, bit: u8) {
        let flags = self.get_interrupt_flags();

        self.write_byte(INTERRUPT_FLAG, flags & !bit);
    }

    fn enable_interrupt_request(&mut self, bit: u8) {
        let flags = self.get_interrupt_flags();

        self.write_byte(INTERRUPT_FLAG, flags | bit);
    }
}

fn get_source_from_bits(bits: u8) -> Option<InterruptSource> {
    match bits {
        b if b & INTERRUPT_VBLANK_BIT != 0 => Some(InterruptSource::VBLANK),
        b if b & INTERRUPT_STAT_BIT != 0 => Some(InterruptSource::STAT),
        b if b & INTERRUPT_TIMER_BIT != 0 => Some(InterruptSource::TIMER),
        b if b & INTERRUPT_SERIAL_BIT != 0 => Some(InterruptSource::SERIAL),
        b if b & INTERRUPT_JOYPAD_BIT != 0 => Some(InterruptSource::JOYPAD),
        _ => None,
    }
}

impl CPU {
    pub fn handle_interrupts(&mut self) -> u8 {
        if !matches!(self.interrupt_state, InterruptState::Enabled) {
            return 0;
        }

        let i_enabled = self.bus.get_interrupt_enabled();
        let i_flags = self.bus.get_interrupt_enabled();
        let bits = i_enabled & i_flags;

        let Some(interrupt_source) = get_source_from_bits(bits) else {
            return 0;
        };

        self.execute_handler(interrupt_source);

        20
    }

    fn execute_handler(&mut self, interrupt_source: InterruptSource) {
        self.interrupt_state = InterruptState::Disabled;
        self.bus.clear_interrupt_source(&interrupt_source);

        eprintln!(
            "Handling interrupt. Leaving {:04X} and jumping to {:04X}",
            self.registers.pc, interrupt_source as u16
        );

        self.call_address(interrupt_source as u16)
    }
}
