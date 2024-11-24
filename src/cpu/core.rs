use super::instructions::Executable;
use super::memory::*;
use super::opcodes::get_instruction;
use super::registers::*;

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

#[derive(Default, Clone, Copy)]
pub struct CPU {
    pub(crate) registers: Registers,
    pub(crate) bus: MemoryBus,
    pub(crate) clock: Clock,
    last_timer_update: u64,
    pub(crate) ime: bool,
    pub(crate) is_halted: bool,
    pub(crate) current_opcode: u8,
}

impl CPU {
    fn read_tac(&self) -> u8 {
        self.bus.memory[0xFF07]
    }

    fn is_timer_enabled(&self) -> bool {
        (self.read_tac() & 0x04) != 0
    }

    fn get_timer_t_frequency(&self) -> u64 {
        const TIMER_FREQUENCIES: [u64; 4] = [256, 4, 16, 64];
        TIMER_FREQUENCIES[(self.read_tac() & 0x03) as usize]
    }

    fn read_tma(&self) -> u8 {
        self.bus.memory[0xFF06]
    }

    fn read_tima(&self) -> u8 {
        self.bus.memory[0xFF05]
    }

    fn write_tima(&mut self, value: u8) {
        self.bus.memory[0xFF05] = value;
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

    pub fn boot_rom(&mut self, boot_rom: &[u8]) {
        let mut i = 0;
        for byte in boot_rom {
            self.bus.memory[i] = *byte;
            i += 1;
        }
    }

    pub fn step(&mut self) {
        let instruction_data = self.fetch();
        self.current_opcode = instruction_data.opcode;

        let (instruction, bytes) = get_instruction(&instruction_data);
        self.registers.pc += bytes;

        let t_cycles = instruction.execute(self);
        self.clock.increment(t_cycles);
        self.update_timers();

        // TODO: update ppu state
        // TODO: handle interrupts
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
