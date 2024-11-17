#![allow(dead_code)]

use super::registers::*;
use crate::cpu::CPU;

pub enum Instruction {
    Add(ADD),
    Adc(ADC),
    Ld(LD),
    Invalid(u8),
}

pub(crate) trait Executable {
    fn execute(&self, cpu: &mut CPU);
}

impl Executable for Instruction {
    fn execute(&self, cpu: &mut CPU) {
        match self {
            Self::Add(instruction) => instruction.execute(cpu),
            Self::Adc(instruction) => instruction.execute(cpu),
            Self::Ld(instruction) => instruction.execute(cpu),
            Self::Invalid(opcode) => todo!(), // freeze and dump out opcode for debugging,
        }
    }
}

pub(crate) enum ByteTarget {
    Constant(u8),
    Register8(R8),
    /// ADD A,[HL] -> Read the byte from the 16-Bit address stored in the HL register
    HLAddress,
}

pub(crate) enum WordTarget {
    /// ADD HL,R16: Adds any 16 bit register to the HL register
    Register16(R16),
    SP,
}

impl CPU {
    /// Common access
    fn read_bytetarget(&self, target: &ByteTarget) -> u8 {
        match target {
            ByteTarget::Constant(value) => *value,
            ByteTarget::Register8(register) => self.read_r8(register),
            ByteTarget::HLAddress => self.bus.read_byte(self.read_hl()),
        }
    }
}

pub(crate) enum ADD {
    Byte(ByteTarget),
    Word(WordTarget),
    StackPointer(i8),
}

impl Executable for ADD {
    fn execute(&self, cpu: &mut CPU) {
        match self {
            ADD::Byte(target) => {
                let value = cpu.read_bytetarget(target);
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
                    WordTarget::Register16(register) => cpu.read_r16(register),
                    WordTarget::SP => cpu.registers.sp,
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

pub(crate) enum LD {
    // LD A,[HLI]
    // LD A,[HLD]
    // LD A,[r16]
    // LD A,[n16]
    LoadToA(R16Mem),
    // LD r8,r8
    // LD r8,n8
    // LD r8,[HL]
    LoadToR8(R8, ByteTarget),
    // LD r16,n16
    LoadToR16(R16, u16),
    // LD SP,n16
    LoadToSP(u16),
    // LD SP,HL
    LoadHLToSp,
    // LD HL,SP+e8
    LoadSPToHL(i8),
    // LD [r16],A
    // LD [n16],A
    // LD [HLI],A
    // LD [HLD],A
    StoreA(R16Mem),
    // LD [HL],r8
    StoreHLRegister(R8),
    // LD [HL],n8
    StoreHLConstant(u8),
    // LD [n16],SP
    StoreSP(u16),
}

impl Executable for LD {
    fn execute(&self, cpu: &mut CPU) {
        match self {
            LD::LoadToA(target) => {
                cpu.registers.a = cpu.read_from(target);
            }
            LD::LoadToR8(register, target) => {
                let value = cpu.read_bytetarget(target);
                cpu.write_r8(register, value);
            }
            LD::LoadToR16(register, value) => {
                cpu.write_r16(register, *value);
            }
            LD::LoadToSP(value) => cpu.registers.sp = *value,
            LD::LoadHLToSp => cpu.registers.sp = cpu.read_hl(),
            LD::LoadSPToHL(offset) => {
                let (sp, _) = cpu.registers.sp.overflowing_add_signed((*offset).into());
                cpu.write_hl(sp);
            }
            LD::StoreA(target) => cpu.store_at(target, cpu.registers.a),
            LD::StoreHLRegister(register) => {
                let address = cpu.read_hl();
                cpu.bus.write_byte(address, cpu.read_r8(register))
            }
            LD::StoreHLConstant(value) => {
                let address = cpu.read_hl();
                cpu.bus.write_byte(address, *value)
            }
            LD::StoreSP(value) => {
                let sp = cpu.registers.sp;
                cpu.bus.write_byte(*value, (sp & 0xFF) as u8);
                cpu.bus.write_byte(*value + 1, (sp >> 8) as u8)
            }
        };
    }
}

pub(crate) enum LDH {
    // LDH [n16],A
    StoreConstant(),
    // LDH [C],A
    StoreOffset(),
    // LDH A,[n16]
    LoadConstant(),
    // LDH A,[C]
    LoadOffset(),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_constant_and_flags() {
        let mut cpu = CPU::default();

        assert_eq!(cpu.registers.a, 0);

        Instruction::Add(ADD::Byte(ByteTarget::Constant(0xFF))).execute(&mut cpu);

        assert_eq!(cpu.registers.a, 0xFF);
        assert_eq!(cpu.registers.f.zero, false);
        assert_eq!(cpu.registers.f.negative, false);
        assert_eq!(cpu.registers.f.carry, false);
        assert_eq!(cpu.registers.f.half_carry, false);

        Instruction::Add(ADD::Byte(ByteTarget::Constant(1))).execute(&mut cpu);

        assert_eq!(cpu.registers.a, 0);
        assert_eq!(cpu.registers.f.zero, true);
        assert_eq!(cpu.registers.f.negative, false);
        assert_eq!(cpu.registers.f.carry, true);
        assert_eq!(cpu.registers.f.half_carry, true);

        Instruction::Add(ADD::Byte(ByteTarget::Constant(0x0F))).execute(&mut cpu);

        assert_eq!(cpu.registers.a, 0x0F);
        assert_eq!(cpu.registers.f.zero, false);
        assert_eq!(cpu.registers.f.negative, false);
        assert_eq!(cpu.registers.f.carry, false);
        assert_eq!(cpu.registers.f.half_carry, false);

        Instruction::Add(ADD::Byte(ByteTarget::Constant(1))).execute(&mut cpu);

        assert_eq!(cpu.registers.a, 0x10);
        assert_eq!(cpu.registers.f.zero, false);
        assert_eq!(cpu.registers.f.negative, false);
        assert_eq!(cpu.registers.f.carry, false);
        assert_eq!(cpu.registers.f.half_carry, true);
    }
}
