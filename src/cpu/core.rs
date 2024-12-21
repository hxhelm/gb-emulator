use super::instructions::Executable;
use super::opcodes::get_instruction;
use super::registers::*;
use crate::memory::bus::Bus;

#[derive(Default, Clone, Copy)]
pub struct Clock {
    pub(crate) t: u64,
    pub(crate) m: u64,
}

impl Clock {
    fn increment(&mut self, t_cycles: u8) {
        self.t = self.m.wrapping_add(t_cycles.into());
        self.m = self.t / 4;
    }
}

pub(crate) struct InstructionData {
    pub(super) opcode: u8,
    pub(super) param1: u8,
    pub(super) param2: u8,
}

#[derive(Default, Clone, Copy)]
pub struct CPU {
    pub(crate) registers: Registers,
    pub(crate) bus: Bus,
    pub(crate) clock: Clock,
    last_timer_update: u64,
    pub(crate) ime: bool,
    pub(crate) is_halted: bool,
    pub(crate) current_opcode: u8,
}

impl CPU {
    pub(crate) fn push_to_stack(&mut self, value: u16) {
        self.registers.sp = self.registers.sp.wrapping_sub(2);
        self.bus.write_word(self.registers.sp, value);
    }

    pub(crate) fn pop_from_stack(&mut self) -> u16 {
        let result = self.bus.read_word(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(2);
        result
    }

    fn read_tac(&self) -> u8 {
        self.bus.read_byte(0xFF07)
    }

    fn is_timer_enabled(&self) -> bool {
        (self.read_tac() & 0x04) != 0
    }

    fn get_timer_t_frequency(&self) -> u64 {
        const TIMER_FREQUENCIES: [u64; 4] = [256, 4, 16, 64];
        TIMER_FREQUENCIES[(self.read_tac() & 0x03) as usize]
    }

    fn read_tma(&self) -> u8 {
        self.bus.read_byte(0xFF06)
    }

    fn read_tima(&self) -> u8 {
        self.bus.read_byte(0xFF05)
    }

    fn write_tima(&mut self, value: u8) {
        self.bus.write_byte(0xFF05, value);
    }

    fn update_timers(&mut self) {
        if !self.is_timer_enabled() {
            return;
        }

        let timer_frequency = self.get_timer_t_frequency();

        let timer_diff = self.clock.t.saturating_sub(self.last_timer_update);
        if timer_diff >= timer_frequency {
            let increments = timer_diff / timer_frequency;
            let (tima, did_overflow) = self.read_tima().overflowing_add(increments as u8);

            if did_overflow {
                self.write_tima(self.read_tma());
                // TODO: trigger interrupt
            } else {
                self.write_tima(tima);
            }

            self.last_timer_update += increments * timer_frequency;
        }
    }

    pub fn load_cartridge(&mut self, rom: &[u8]) {
        self.bus.write_cartridge(rom);
    }

    pub fn load_boot_rom(&mut self, boot_rom: &[u8]) {
        self.bus.write_boot_rom(boot_rom);
    }

    // TODO: handle out of bound fetch
    pub(crate) fn fetch(&self) -> InstructionData {
        InstructionData {
            opcode: self.bus.read_byte(self.registers.pc),
            param1: self.bus.read_byte(self.registers.pc + 1),
            param2: self.bus.read_byte(self.registers.pc + 2),
        }
    }

    pub fn step(&mut self) -> u8 {
        let instruction_data = self.fetch();
        self.current_opcode = instruction_data.opcode;

        let (instruction, bytes) = get_instruction(&instruction_data);
        self.registers.pc += bytes;

        let t_cycles = instruction.execute(self);
        self.clock.increment(t_cycles);
        self.update_timers();

        // test rom serial output
        if self.bus.read_byte(0xFF02) == 0x81 {
            eprintln!("{}", self.bus.read_byte(0xFF01));
            self.bus.write_byte(0xFF01, 0x00);
        }

        // TODO: handle interrupts

        t_cycles
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_16_bit_register() {
        let mut cpu = CPU::default();

        cpu.write_bc(0x0B0C);
        cpu.write_de(0xD0E0);
        cpu.write_hl(0xFF11);

        assert_eq!(cpu.registers.b, 0xB);
        assert_eq!(cpu.registers.c, 0xC);
        assert_eq!(cpu.registers.d, 0xD0);
        assert_eq!(cpu.registers.e, 0xE0);
        assert_eq!(cpu.registers.h, 0xFF);
        assert_eq!(cpu.registers.l, 0x11);
    }

    #[test]
    fn read_16_bit_register() {
        let mut cpu = CPU::default();

        cpu.registers.b = 0xB0;
        cpu.registers.c = 0xC0;
        cpu.registers.d = 0x0D;
        cpu.registers.e = 0x0E;
        cpu.registers.h = 0x11;
        cpu.registers.l = 0xFF;

        assert_eq!(cpu.read_bc(), 0xB0C0);
        assert_eq!(cpu.read_de(), 0x0D0E);
        assert_eq!(cpu.read_hl(), 0x11FF);
    }
}
