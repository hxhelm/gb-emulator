#![allow(dead_code)]

use super::instructions::Executable;
use super::opcodes::get_instruction;
use super::CPU;
use std::usize;

const MEMORY_BUS_SIZE: usize = 0xFFFF;

pub(crate) struct MemoryBus {
    memory: [u8; MEMORY_BUS_SIZE],
}

impl Default for MemoryBus {
    fn default() -> Self {
        Self {
            memory: [Default::default(); MEMORY_BUS_SIZE],
        }
    }
}

impl MemoryBus {
    pub fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn read_byte_at_offset(&self, offset: u8) -> u8 {
        let address = 0xFF00 + offset as u16;
        self.read_byte(address)
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        self.memory[address as usize] = byte;
    }

    pub fn write_byte_at_offset(&mut self, offset: u8, byte: u8) {
        let address = 0xFF00 + offset as u16;
        self.write_byte(address, byte)
    }

    pub fn write_word(&mut self, address: u16, word: u16) {
        let [lsb, msb] = word.to_le_bytes();
        self.memory[address as usize] = lsb;
        self.memory[(address + 1) as usize] = msb;
    }

    pub fn read_word(&mut self, address: u16) -> u16 {
        let lsb = self.memory[address as usize];
        let msb = self.memory[(address + 1) as usize];

        u16::from_le_bytes([lsb, msb])
    }
}

const FETCH_BYTE_COUNT: usize = 3;
pub(crate) struct InstructionData {
    pub(super) opcode: u8,
    pub(super) param1: u8,
    pub(super) param2: u8,
}

impl CPU {
    pub(crate) fn push_to_stack(&mut self, value: u16) {
        self.registers.sp -= 2;
        self.bus.write_word(self.registers.sp, value);
    }

    pub(crate) fn pop_from_stack(&mut self) -> u16 {
        let result = self.bus.read_word(self.registers.sp);
        self.registers.sp += 2;
        result
    }

    // TODO: handle out of bound fetch
    fn fetch(&self) -> InstructionData {
        let start = self.registers.pc as usize;
        let end = start + FETCH_BYTE_COUNT;

        let slice = &self.bus.memory[start..end];
        assert_eq!(slice.len(), FETCH_BYTE_COUNT);

        let [opcode, param1, param2] = slice else {
            todo!();
        };

        InstructionData {
            opcode: *opcode,
            param1: *param1,
            param2: *param2,
        }
    }

    fn step(&mut self) {
        let instruction_data = self.fetch();

        let (instruction, bytes) = get_instruction(&instruction_data);

        self.registers.pc += bytes;

        instruction.execute(self);
    }
}
