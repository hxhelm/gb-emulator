use super::instructions::Executable;
use super::opcodes::get_instruction;
use super::registers::*;
use super::timers::Clock;
use crate::memory::bus::Bus;

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
    pub(crate) last_timer_update: u64,
    pub(crate) ime: bool,
    pub(crate) is_halted: bool,
    pub(crate) current_opcode: u8,
}

impl CPU {
    // TODO: make boot rom optional? look into expected state after boot rom
    pub fn init(boot_rom: &[u8], cartridge_contents: Option<&[u8]>) -> Self {
        let mut cpu = Self::default();

        // TODO: support bigger cartridges with memory bank https://gbdev.io/pandocs/MBCs.html#mbcs
        if let Some(rom) = cartridge_contents {
            cpu.load_cartridge(rom);
        }

        cpu.load_boot_rom(boot_rom);

        cpu
    }

    pub(crate) fn push_to_stack(&mut self, value: u16) {
        self.registers.sp = self.registers.sp.wrapping_sub(2);
        self.bus.write_word(self.registers.sp, value);
    }

    pub(crate) fn pop_from_stack(&mut self) -> u16 {
        let result = self.bus.read_word(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(2);
        result
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

        // if self.registers.pc >= 0x100 {
        //     self.log_state();
        // }

        let (instruction, bytes) = get_instruction(&instruction_data);
        self.registers.pc += bytes;

        let t_cycles = instruction.execute(self);
        self.clock.increment(t_cycles);
        self.update_timers();

        self.print_serial_output();

        // TODO: handle interrupts

        t_cycles
    }

    fn print_serial_output(&mut self) {
        // test rom serial output
        if self.bus.read_byte(0xFF02) == 0x81 {
            let character = self.bus.read_byte(0xFF01);
            if character != 0x00 {
                eprint!("{}", character as char);
                self.bus.write_byte(0xFF01, 0x00);
            }
        }
    }

    #[allow(unused)]
    pub fn log_state(&self) {
        eprint!("A: {:02X} ", self.registers.a);
        let f: u8 = (&self.registers.f).into();
        eprint!("F: {:02X} ", f);
        eprint!("B: {:02X} ", self.registers.b);
        eprint!("C: {:02X} ", self.registers.c);
        eprint!("D: {:02X} ", self.registers.d);
        eprint!("E: {:02X} ", self.registers.e);
        eprint!("H: {:02X} ", self.registers.h);
        eprint!("L: {:02X} ", self.registers.l);

        eprint!("SP: {:004X} ", self.registers.sp);

        eprint!("PC: 00:{:004X} ", self.registers.pc);

        let bytes = self.bus.read_range(self.registers.pc, 4);

        assert_eq!(bytes.len(), 4);

        eprint!(
            "({:02X} {:02X} {:02X} {:02X})\n",
            bytes[0], bytes[1], bytes[2], bytes[3]
        );
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
