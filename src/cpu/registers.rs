#![allow(dead_code)]
/// Defines common register groups foundin instructions and provides some helper functions
use crate::cpu::CPU;

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
}

pub(crate) trait R16MemOps {
    fn read_from(&mut self, register: &R16Mem) -> u8;
    fn store_at(&mut self, register: &R16Mem, value: u8);
}

impl R16MemOps for CPU {
    fn read_from(&mut self, register: &R16Mem) -> u8 {
        let address = self.read_r16m(register);

        return self.bus.read_byte(address);
    }

    fn store_at(&mut self, register: &R16Mem, value: u8) {
        let address = self.read_r16m(register);

        self.bus.write_byte(address, value);
    }
}
