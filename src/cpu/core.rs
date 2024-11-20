#![allow(dead_code)]

use super::instructions::Executable;
use super::memory::*;
use super::opcodes::get_instruction;
use std::convert::From;
use std::thread::sleep;
use std::time;

#[derive(Default)]
pub(crate) struct Flags {
    pub(crate) zero: bool,
    pub(crate) negative: bool,
    pub(crate) half_carry: bool,
    pub(crate) carry: bool,
}

const ZERO_FLAG_BYTE_POSITION: u8 = 7;
const NEGATIVE_FLAG_BYTE_POSITION: u8 = 6;
const HALF_CARRY_FLAG_BYTE_POSITION: u8 = 5;
const CARRY_FLAG_BYTE_POSITION: u8 = 4;

impl From<&Flags> for u8 {
    fn from(flags: &Flags) -> u8 {
        (if flags.zero { 1 } else { 0 }) << ZERO_FLAG_BYTE_POSITION
            | (if flags.negative { 1 } else { 0 }) << NEGATIVE_FLAG_BYTE_POSITION
            | (if flags.half_carry { 1 } else { 0 }) << HALF_CARRY_FLAG_BYTE_POSITION
            | (if flags.carry { 1 } else { 0 }) << CARRY_FLAG_BYTE_POSITION
    }
}

impl From<u8> for Flags {
    fn from(value: u8) -> Self {
        let zero = ((value >> ZERO_FLAG_BYTE_POSITION) & 0b1) != 0;
        let negative = ((value >> NEGATIVE_FLAG_BYTE_POSITION) & 0b1) != 0;
        let half_carry = ((value >> HALF_CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;
        let carry = ((value >> CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;

        Flags {
            zero,
            negative,
            half_carry,
            carry,
        }
    }
}

#[derive(Default)]
pub(crate) struct Registers {
    pub(crate) a: u8,
    pub(crate) b: u8,
    pub(crate) c: u8,
    pub(crate) d: u8,
    pub(crate) e: u8,
    pub(crate) f: Flags,
    pub(crate) h: u8,
    pub(crate) l: u8,
    pub(crate) sp: u16,
    pub(crate) pc: u16,
}

#[derive(Default)]
pub struct CPU {
    pub(crate) registers: Registers,
    pub(crate) bus: MemoryBus,
    pub(crate) ime: bool,
    pub(crate) is_halted: bool,
}

impl CPU {
    pub fn read_af(&self) -> u16 {
        let flags: u8 = u8::from(&self.registers.f);
        (self.registers.a as u16) << 8 | flags as u16
    }

    pub fn read_bc(&self) -> u16 {
        (self.registers.b as u16) << 8 | self.registers.c as u16
    }

    pub fn write_bc(&mut self, value: u16) {
        [self.registers.b, self.registers.c] = value.to_be_bytes();
    }

    pub fn read_de(&self) -> u16 {
        (self.registers.d as u16) << 8 | self.registers.e as u16
    }

    pub fn write_de(&mut self, value: u16) {
        [self.registers.d, self.registers.e] = value.to_be_bytes();
    }

    pub fn read_hl(&self) -> u16 {
        (self.registers.h as u16) << 8 | self.registers.l as u16
    }

    pub fn write_hl(&mut self, value: u16) {
        [self.registers.h, self.registers.l] = value.to_be_bytes();
    }

    pub fn read_hl_ptr(&self) -> u8 {
        self.bus.read_byte(self.read_hl())
    }

    pub fn write_hl_ptr(&mut self, value: u8) {
        self.bus.write_byte(self.read_hl(), value);
    }

    pub fn call_address(&mut self, address: u16) {
        self.push_to_stack(self.registers.pc);
        self.registers.pc = address;
    }

    fn step(&mut self) {
        let instruction_data = self.fetch();

        let (instruction, bytes) = get_instruction(&instruction_data);
        let pre_instruction_pc = self.registers.pc;

        println!(
            "Read opcode {:02X}, len: {}B, PC: {:04X}",
            instruction_data.opcode, bytes, pre_instruction_pc
        );

        instruction.execute(self);

        if self.registers.pc == pre_instruction_pc {
            self.registers.pc += bytes;
        }

        sleep(time::Duration::from_millis(100))
    }

    pub fn boot(&mut self, boot_rom: &[u8]) {
        let mut i = 0;
        for byte in boot_rom {
            self.bus.memory[i] = *byte;
            i += 1;
        }

        while !self.is_halted {
            self.step();
        }
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
