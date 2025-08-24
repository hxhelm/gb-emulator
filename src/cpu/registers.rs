#![allow(dead_code)]
/// Defines common register groups found in instructions and provides some helper functions
use crate::cpu::CPU;
use std::convert::From;

#[derive(Default, Clone, Copy)]
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

#[derive(Clone, Copy)]
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

impl Default for Registers {
    fn default() -> Self {
        Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: Flags::default(),
            h: 0,
            l: 0,
            sp: 0xFFFE,
            pc: 0,
        }
    }
}

pub(crate) enum R8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

pub(crate) enum R16 {
    BC,
    DE,
    HL,
}

pub(crate) enum R16Mem {
    BC,
    DE,
    HLI,
    HLD,
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
        self.read_byte(self.read_hl())
    }

    pub fn write_hl_ptr(&mut self, value: u8) {
        self.write_byte(self.read_hl(), value);
    }

    pub(crate) fn read_r8(&self, register: &R8) -> u8 {
        match register {
            R8::A => self.registers.a,
            R8::B => self.registers.b,
            R8::C => self.registers.c,
            R8::D => self.registers.d,
            R8::E => self.registers.e,
            R8::H => self.registers.h,
            R8::L => self.registers.l,
        }
    }

    pub(crate) fn write_r8(&mut self, register: &R8, value: u8) {
        match register {
            R8::A => self.registers.a = value,
            R8::B => self.registers.b = value,
            R8::C => self.registers.c = value,
            R8::D => self.registers.d = value,
            R8::E => self.registers.e = value,
            R8::H => self.registers.h = value,
            R8::L => self.registers.l = value,
        };
    }

    pub(crate) fn read_r16(&self, register: &R16) -> u16 {
        match register {
            R16::BC => self.read_bc(),
            R16::DE => self.read_de(),
            R16::HL => self.read_hl(),
        }
    }

    pub(crate) fn write_r16(&mut self, register: &R16, value: u16) {
        match register {
            R16::BC => self.write_bc(value),
            R16::DE => self.write_de(value),
            R16::HL => self.write_hl(value),
        }
    }

    fn read_r16m(&mut self, register: &R16Mem) -> u16 {
        match register {
            R16Mem::BC => self.read_bc(),
            R16Mem::DE => self.read_de(),
            R16Mem::HLI => {
                let hl = self.read_hl();
                self.write_hl(hl.overflowing_add(1).0);
                hl
            }
            R16Mem::HLD => {
                let hl = self.read_hl();
                self.write_hl(hl.overflowing_sub(1).0);
                hl
            }
        }
    }

    pub(crate) fn read_from(&mut self, register: &R16Mem) -> u8 {
        let address = self.read_r16m(register);

        self.bus.read_byte(address)
    }

    pub(crate) fn store_at(&mut self, register: &R16Mem, value: u8) {
        let address = self.read_r16m(register);

        self.write_byte(address, value);
    }
}
