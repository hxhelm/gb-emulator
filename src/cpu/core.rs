use super::instructions::Executable;
use super::interrupts::{HaltState, InterruptState};
use super::opcodes::get_instruction;
use super::registers::*;
use super::timers::Clock;
use crate::memory::bus::{Bus, SERIAL_TRANSFER_CONTROL, SERIAL_TRANSFER_DATA};

#[derive(Default, Clone, Copy)]
pub(crate) struct InstructionData {
    pub(crate) opcode: u8,
    pub(super) param1: u8,
    pub(super) param2: u8,
}

#[derive(Default, Clone)]
pub struct CPU {
    pub(crate) registers: Registers,
    pub(crate) bus: Bus,
    pub(crate) current_instruction: InstructionData,
    /// Halt state and halt bug check
    pub(crate) halt_state: HaltState,
    /// Timers
    pub(super) clock: Clock,
    pub(super) last_timer_update: u64,
    /// Interrupt handling
    pub(crate) interrupt_state: InterruptState,
}

impl CPU {
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
                log::debug!("{}", character as char);
                self.bus.write_byte(SERIAL_TRANSFER_DATA, 0x00);
            }
        }
    }

    #[allow(unused)]
    pub fn log_state(&self) {
        log::debug!("A: {:02X} ", self.registers.a);
        let f: u8 = (&self.registers.f).into();
        log::debug!("F: {:02X} ", f);
        log::debug!("B: {:02X} ", self.registers.b);
        log::debug!("C: {:02X} ", self.registers.c);
        log::debug!("D: {:02X} ", self.registers.d);
        log::debug!("E: {:02X} ", self.registers.e);
        log::debug!("H: {:02X} ", self.registers.h);
        log::debug!("L: {:02X} ", self.registers.l);

        log::debug!("SP: {:004X} ", self.registers.sp);

        log::debug!("PC: 00:{:004X} ", self.registers.pc);

        log::debug!(
            "({:02X} {:02X} {:02X} {:02X})\n",
            self.bus.read_byte(self.registers.pc),
            self.bus.read_byte(self.registers.pc + 1),
            self.bus.read_byte(self.registers.pc + 2),
            self.bus.read_byte(self.registers.pc + 3),
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
