use super::instructions::Executable;
use super::interrupts::{HaltState, InterruptState};
use super::opcodes::get_instruction;
use super::registers::*;
use super::serial::{SERIAL_TRANSFER_CONTROL, SERIAL_TRANSFER_DATA};
use super::timers::Clock;
use crate::memory::bus::Bus;

#[derive(Default, Clone, Copy)]
pub(crate) struct InstructionData {
    pub(crate) opcode: u8,
    pub(super) param1: u8,
    pub(super) param2: u8,
}

#[derive(Default, Clone, Copy)]
pub struct CPU {
    pub(crate) registers: Registers,
    pub(crate) bus: Bus,
    pub(crate) current_instruction: InstructionData,
    /// Halt state and halt bug check
    pub(crate) halt_state: HaltState,
    /// Timers
    pub(crate) clock: Clock,
    pub(crate) last_timer_update: u64,
    /// Interrupt handling
    pub(crate) interrupt_state: InterruptState,
}

impl CPU {
    // TODO: make boot rom optional? look into expected state after boot rom
    pub fn init(boot_rom: Option<&[u8]>, cartridge_contents: &[u8]) -> Self {
        let mut cpu = Self::default();

        // TODO: support bigger cartridges with memory bank https://gbdev.io/pandocs/MBCs.html#mbcs
        cpu.load_cartridge(cartridge_contents);

        match boot_rom {
            Some(rom) => cpu.load_boot_rom(rom),
            None => cpu.init_boot_handoff(),
        }

        cpu
    }

    // TODO: handle out of bound fetch
    fn fetch(&mut self) -> InstructionData {
        match self.halt_state {
            HaltState::HaltBug => {
                self.halt_state = HaltState::NotHalted;

                InstructionData {
                    opcode: self.bus.read_byte(self.registers.pc),
                    param1: self.bus.read_byte(self.registers.pc),
                    param2: self.bus.read_byte(self.registers.pc + 1),
                }
            }
            _ => InstructionData {
                opcode: self.bus.read_byte(self.registers.pc),
                param1: self.bus.read_byte(self.registers.pc + 1),
                param2: self.bus.read_byte(self.registers.pc + 2),
            },
        }
    }

    pub fn step(&mut self) -> u8 {
        self.current_instruction = self.fetch();

        if matches!(self.halt_state, HaltState::Halted) {
            // keep incrementing timers during halted state by base cycles of 4
            let cycles = self.handle_halted_interrupts() + 4;
            self.update_timers(cycles);
            return cycles;
        }

        let (instruction, bytes) = get_instruction(&self.current_instruction);

        if !matches!(self.halt_state, HaltState::HaltBug) {
            self.registers.pc += bytes;
        }

        let instruction_cycles = instruction.execute(self);
        self.update_timers(instruction_cycles);

        // Handle Interrupt Enable requested by EI instruction, which is delayed by one instruction
        if matches!(self.interrupt_state, InterruptState::EnableRequested)
            && self.current_instruction.opcode != 0xD9
        {
            self.interrupt_state = InterruptState::Enabled;
        }

        let interrupt_cycles = self.handle_interrupts();
        self.update_timers(interrupt_cycles);

        self.print_serial_output();

        instruction_cycles + interrupt_cycles
    }

    fn print_serial_output(&mut self) {
        // test rom serial output
        if self.bus.read_byte(SERIAL_TRANSFER_CONTROL) == 0x81 {
            let character = self.bus.read_byte(SERIAL_TRANSFER_DATA);
            if character != 0x00 {
                eprint!("{}", character as char);
                self.bus.write_byte(SERIAL_TRANSFER_DATA, 0x00);
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
