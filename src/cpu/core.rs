use super::dma::DmaState;
use super::instructions::Executable;
use super::interrupts::{HaltState, InterruptState};
use super::opcodes::get_instruction;
use super::registers::*;
use super::timers::Clock;
use crate::memory::bus::{Bus, DMA_START, SERIAL_TRANSFER_CONTROL, SERIAL_TRANSFER_DATA};

#[derive(Default, Clone, Copy)]
pub(crate) struct InstructionData {
    pub(crate) opcode: u8,
    pub(super) param1: u8,
    pub(super) param2: u8,
}

#[derive(Clone)]
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
    /// DMA State
    pub(crate) dma_state: DmaState,
}

impl CPU {
    fn from_cardridge(cartridge_contents: &[u8]) -> Self {
        Self {
            registers: Registers::default(),
            bus: Bus::from_cartridge(cartridge_contents),
            current_instruction: InstructionData::default(),
            halt_state: HaltState::default(),
            clock: Clock::default(),
            last_timer_update: 0,
            interrupt_state: InterruptState::default(),
            dma_state: DmaState::Inactive,
        }
    }

    pub fn init(boot_rom: Option<&[u8]>, cartridge_contents: &[u8]) -> Self {
        let mut cpu = Self::from_cardridge(cartridge_contents);

        match boot_rom {
            Some(rom) => cpu.load_boot_rom(rom),
            None => cpu.init_boot_handoff(),
        }

        cpu
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.bus.read_byte(address)
    }

    // TODO: wrapper for DMA detection
    pub fn write_byte(&mut self, address: u16, byte: u8) {
        self.bus.write_byte(address, byte);

        if address == DMA_START {
            self.dma_state = DmaState::init_dma_transfer(byte);
        }
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
        if !matches!(self.dma_state.step(&mut self.bus), DmaState::Inactive) {
            return 4;
        }

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

        // self.log_state();

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
                log::info!("{}", character as char);
                self.bus.write_byte(SERIAL_TRANSFER_DATA, 0x00);
            }
        }
    }

    #[allow(unused)]
    pub fn log_state(&self) {
        log::debug!(
            target: "instruction",
            "A: {:02X} F: {:02X} B: {:02X} C: {:02X} D: {:02X} E: {:02X} H: {:02X} L: {:02X} SP: {:04X} PC: 00:{:04X} ({:02X} {:02X} {:02X} {:02X})",
            self.registers.a,
            u8::from(&self.registers.f),
            self.registers.b,
            self.registers.c,
            self.registers.d,
            self.registers.e,
            self.registers.h,
            self.registers.l,
            self.registers.sp,
            self.registers.pc,
            self.bus.read_byte(self.registers.pc),
            self.bus.read_byte(self.registers.pc + 1),
            self.bus.read_byte(self.registers.pc + 2),
            self.bus.read_byte(self.registers.pc + 3),
        );
    }
}
