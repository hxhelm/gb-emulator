#![allow(dead_code)]

use self::operators::*;
use crate::cpu::CPU;

mod operators {
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
}

enum Instruction {
    Add(ADD),
    Adc(ADC),
}

impl Executable for Instruction {
    fn execute(&self, cpu: &mut CPU) {
        match self {
            Self::Add(instruction) => instruction.execute(cpu),
            Self::Adc(instruction) => instruction.execute(cpu),
        }
    }
}

pub(crate) trait Executable {
    fn execute(&self, cpu: &mut CPU);
}

pub(crate) enum Target8 {
    Constant(u8),
    Register8(R8),
    /// ADD A,[HL] -> Read the byte from the 16-Bit address stored in the HL register
    HLAddress,
}

pub(crate) enum Target16 {
    /// ADD HL,R16: Adds any 16 bit register to the HL register
    Register16(R16),
    SP,
}

pub(crate) enum ADD {
    Byte(Target8),
    Word(Target16),
    StackPointer(i8),
}

impl Executable for ADD {
    fn execute(&self, cpu: &mut CPU) {
        match self {
            ADD::Byte(target) => {
                let value = match target {
                    Target8::Constant(value) => *value,
                    Target8::Register8(register) => cpu.read_r8(register),
                    Target8::HLAddress => cpu.bus.read_byte(cpu.read_hl()),
                };

                let (result, did_overflow) = cpu.registers.a.overflowing_add(value);

                cpu.registers.f.zero = result == 0;
                cpu.registers.f.negative = false;
                cpu.registers.f.carry = did_overflow;
                cpu.registers.f.half_carry =
                    (((cpu.registers.a & 0xF) + (value & 0xF)) & 0x10) == 0x10;

                cpu.registers.a = result;
            }
            ADD::Word(target) => {
                let value = match target {
                    Target16::Register16(register) => cpu.read_r16(register),
                    Target16::SP => cpu.registers.sp,
                };

                let (result, did_overflow) = cpu.read_hl().overflowing_add(value);

                cpu.registers.f.negative = false;
                cpu.registers.f.carry = did_overflow;
                cpu.registers.f.half_carry =
                    (((cpu.read_hl() & 0xFFF) + (value & 0xFFF)) & 0x1000) == 0x1000;

                cpu.write_hl(result);
            }
            ADD::StackPointer(offset) => {
                let (result, did_overflow) =
                    cpu.registers.sp.overflowing_add_signed((*offset).into());

                cpu.registers.f.zero = false;
                cpu.registers.f.negative = false;
                cpu.registers.f.carry = did_overflow;

                let sp_u8 = ((cpu.registers.sp & 0xFF00) >> 8) as i8;
                cpu.registers.f.half_carry = (((sp_u8 & 0xF) + (offset & 0xF)) & 0x10) == 0x10;

                cpu.registers.sp = result;
            }
        };
    }
}

pub(crate) enum ADC {
    Constant(u8),
    Register(R8),
    HLAddress,
}

impl Executable for ADC {
    fn execute(&self, cpu: &mut CPU) {
        let carry_val: u8 = if cpu.registers.f.carry { 1 } else { 0 };
        let value = match self {
            ADC::Constant(value) => *value,
            ADC::Register(register) => cpu.read_r8(register),
            ADC::HLAddress => cpu.bus.read_byte(cpu.read_hl()),
        };

        let (value, carry_overflow) = carry_val.overflowing_add(value);
        let (result, did_overflow) = cpu.registers.a.overflowing_add(value);
        cpu.registers.a = result;

        cpu.registers.f.zero = result == 0;
        cpu.registers.f.negative = false;
        cpu.registers.f.carry = did_overflow | carry_overflow;
        cpu.registers.f.half_carry = (cpu.registers.a & 0xF) + (result & 0xF) > 0xF;
    }
}

impl CPU {
    fn read_r8(&self, register: &R8) -> u8 {
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

    fn read_r16(&self, register: &R16) -> u16 {
        match register {
            R16::BC => ((self.registers.b as u16) << 8) | (self.registers.c as u16),
            R16::DE => ((self.registers.d as u16) << 8) | (self.registers.e as u16),
            R16::HL => ((self.registers.h as u16) << 8) | (self.registers.l as u16),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_constant_and_flags() {
        let mut cpu = CPU::default();

        assert_eq!(cpu.registers.a, 0);

        Instruction::Add(ADD::Byte(Target8::Constant(0xFF))).execute(&mut cpu);

        assert_eq!(cpu.registers.a, 0xFF);
        assert_eq!(cpu.registers.f.zero, false);
        assert_eq!(cpu.registers.f.negative, false);
        assert_eq!(cpu.registers.f.carry, false);
        assert_eq!(cpu.registers.f.half_carry, false);

        Instruction::Add(ADD::Byte(Target8::Constant(1))).execute(&mut cpu);

        assert_eq!(cpu.registers.a, 0);
        assert_eq!(cpu.registers.f.zero, true);
        assert_eq!(cpu.registers.f.negative, false);
        assert_eq!(cpu.registers.f.carry, true);
        assert_eq!(cpu.registers.f.half_carry, true);

        Instruction::Add(ADD::Byte(Target8::Constant(0x0F))).execute(&mut cpu);

        assert_eq!(cpu.registers.a, 0x0F);
        assert_eq!(cpu.registers.f.zero, false);
        assert_eq!(cpu.registers.f.negative, false);
        assert_eq!(cpu.registers.f.carry, false);
        assert_eq!(cpu.registers.f.half_carry, false);

        Instruction::Add(ADD::Byte(Target8::Constant(1))).execute(&mut cpu);

        assert_eq!(cpu.registers.a, 0x10);
        assert_eq!(cpu.registers.f.zero, false);
        assert_eq!(cpu.registers.f.negative, false);
        assert_eq!(cpu.registers.f.carry, false);
        assert_eq!(cpu.registers.f.half_carry, true);
    }
}
