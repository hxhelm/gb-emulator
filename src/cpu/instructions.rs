#![allow(dead_code)]

use super::registers::*;
use crate::cpu::CPU;

pub enum Instruction {
    Add(ADD),
    Adc(ADC),
    Sub(SUB),
    Sbc(SBC),
    And(AND),
    Xor(XOR),
    Or(OR),
    Cp(CP),
    Inc(INC),
    Dec(DEC),
    Ld(LD),
    Ldh(LDH),
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
            Self::Sub(instruction) => instruction.execute(cpu),
            Self::Sbc(instruction) => instruction.execute(cpu),
            Self::And(instruction) => instruction.execute(cpu),
            Self::Xor(instruction) => instruction.execute(cpu),
            Self::Or(instruction) => instruction.execute(cpu),
            Self::Cp(instruction) => instruction.execute(cpu),
            Self::Inc(instruction) => instruction.execute(cpu),
            Self::Dec(instruction) => instruction.execute(cpu),
            Self::Ld(instruction) => instruction.execute(cpu),
            Self::Ldh(instruction) => instruction.execute(cpu),
            Self::Invalid(_opcode) => todo!(), // freeze and dump out opcode for debugging,
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
            ByteTarget::HLAddress => self.read_hl_ptr(),
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
                cpu.registers.f.half_carry = half_carry_set_u8(cpu.registers.a, value);

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
                cpu.registers.f.half_carry = half_carry_set_u16(cpu.read_hl(), value);

                cpu.write_hl(result);
            }
            ADD::StackPointer(offset) => {
                let (result, did_overflow) =
                    cpu.registers.sp.overflowing_add_signed((*offset).into());

                cpu.registers.f.zero = false;
                cpu.registers.f.negative = false;
                cpu.registers.f.carry = did_overflow;

                let sp_i8 = ((cpu.registers.sp & 0xFF00) >> 8) as i8;
                cpu.registers.f.half_carry = half_carry_set_i8(sp_i8, *offset);

                cpu.registers.sp = result;
            }
        };
    }
}

pub(crate) struct ADC(pub(crate) ByteTarget);

impl Executable for ADC {
    fn execute(&self, cpu: &mut CPU) {
        let carry_val: u8 = if cpu.registers.f.carry { 1 } else { 0 };
        let value = cpu.read_bytetarget(&self.0);

        let (value, carry_overflow) = carry_val.overflowing_add(value);
        let (result, did_overflow) = cpu.registers.a.overflowing_add(value);

        cpu.registers.f.zero = result == 0;
        cpu.registers.f.negative = false;
        cpu.registers.f.carry = did_overflow | carry_overflow;
        cpu.registers.f.half_carry = half_carry_set_u8(cpu.registers.a, result);

        cpu.registers.a = result;
    }
}

pub(crate) struct SUB(pub(crate) ByteTarget);

impl Executable for SUB {
    fn execute(&self, cpu: &mut CPU) {
        let value = cpu.read_bytetarget(&self.0);
        let (result, did_overflow) = cpu.registers.a.overflowing_sub(value);

        cpu.registers.f.zero = result == 0;
        cpu.registers.f.negative = true;
        cpu.registers.f.carry = did_overflow;
        cpu.registers.f.half_carry = half_carry_set_u8(cpu.registers.a, value);

        cpu.registers.a = result;
    }
}

pub(crate) struct SBC(pub(crate) ByteTarget);

impl Executable for SBC {
    fn execute(&self, cpu: &mut CPU) {
        let carry_val: u8 = if cpu.registers.f.carry { 1 } else { 0 };
        let value = cpu.read_bytetarget(&self.0);

        let (value, carry_overflow) = carry_val.overflowing_sub(value);
        let (result, did_overflow) = cpu.registers.a.overflowing_sub(value);

        cpu.registers.f.zero = result == 0;
        cpu.registers.f.negative = true;
        cpu.registers.f.carry = did_overflow | carry_overflow;
        cpu.registers.f.half_carry = half_carry_set_u8(cpu.registers.a, result);

        cpu.registers.a = result;
    }
}

pub(crate) struct AND(pub(crate) ByteTarget);

impl Executable for AND {
    fn execute(&self, cpu: &mut CPU) {
        let value = cpu.read_bytetarget(&self.0);
        let result = cpu.registers.a & value;

        cpu.registers.f.zero = result == 0;
        cpu.registers.f.negative = false;
        cpu.registers.f.carry = false;
        cpu.registers.f.half_carry = true;

        cpu.registers.a = result;
    }
}

pub(crate) struct XOR(pub(crate) ByteTarget);

impl Executable for XOR {
    fn execute(&self, cpu: &mut CPU) {
        let value = cpu.read_bytetarget(&self.0);
        let result = cpu.registers.a ^ value;

        cpu.registers.f.zero = result == 0;
        cpu.registers.f.negative = false;
        cpu.registers.f.carry = false;
        cpu.registers.f.half_carry = false;

        cpu.registers.a = result;
    }
}

pub(crate) struct OR(pub(crate) ByteTarget);

impl Executable for OR {
    fn execute(&self, cpu: &mut CPU) {
        let value = cpu.read_bytetarget(&self.0);
        let result = cpu.registers.a | value;

        cpu.registers.f.zero = result == 0;
        cpu.registers.f.negative = false;
        cpu.registers.f.carry = false;
        cpu.registers.f.half_carry = false;

        cpu.registers.a = result;
    }
}

pub(crate) struct CP(pub(crate) ByteTarget);

impl Executable for CP {
    fn execute(&self, cpu: &mut CPU) {
        let value = cpu.read_bytetarget(&self.0);
        let (result, did_overflow) = cpu.registers.a.overflowing_sub(value);

        cpu.registers.f.zero = result == 0;
        cpu.registers.f.negative = true;
        cpu.registers.f.carry = did_overflow;
        cpu.registers.f.half_carry = half_carry_set_u8(cpu.registers.a, value);
    }
}

pub(crate) enum INC {
    R8(R8),
    HL,
    R16(R16),
    SP,
}

impl Executable for INC {
    fn execute(&self, cpu: &mut CPU) {
        match self {
            INC::R8(register) => {
                let current = cpu.read_r8(register);
                let (result, _) = current.overflowing_add(1);

                cpu.registers.f.zero = result == 0;
                cpu.registers.f.negative = false;
                cpu.registers.f.half_carry = half_carry_set_u8(current, 1);

                cpu.write_r8(register, result);
            }
            INC::HL => {
                let current = cpu.read_hl_ptr();
                let (result, _) = current.overflowing_add(1);

                cpu.registers.f.zero = result == 0;
                cpu.registers.f.negative = false;
                cpu.registers.f.half_carry = half_carry_set_u8(current, 1);

                cpu.write_hl_ptr(result);
            }
            INC::R16(register) => {
                let (result, _) = cpu.read_r16(register).overflowing_add(1);

                cpu.write_r16(register, result);
            }
            INC::SP => {
                let (result, _) = cpu.registers.sp.overflowing_add(1);

                cpu.registers.sp = result;
            }
        }
    }
}

pub(crate) enum DEC {
    R8(R8),
    HL,
    R16(R16),
    SP,
}

impl Executable for DEC {
    fn execute(&self, cpu: &mut CPU) {
        match self {
            DEC::R8(register) => {
                let current = cpu.read_r8(register);
                let (result, _) = current.overflowing_sub(1);

                cpu.registers.f.zero = result == 0;
                cpu.registers.f.negative = true;
                cpu.registers.f.half_carry = half_carry_set_u8(current, 1);

                cpu.write_r8(register, result);
            }
            DEC::HL => {
                let current = cpu.read_hl_ptr();
                let (result, _) = current.overflowing_sub(1);

                cpu.registers.f.zero = result == 0;
                cpu.registers.f.negative = true;
                cpu.registers.f.half_carry = half_carry_set_u8(current, 1);

                cpu.write_hl_ptr(result);
            }
            DEC::R16(register) => {
                let (result, _) = cpu.read_r16(register).overflowing_sub(1);

                cpu.write_r16(register, result);
            }
            DEC::SP => {
                let (result, _) = cpu.registers.sp.overflowing_sub(1);

                cpu.registers.sp = result;
            }
        }
    }
}

pub(crate) enum LD {
    // LD A,[HLI]
    // LD A,[HLD]
    // LD A,[r16]
    LoadToA(R16Mem),
    // LD A,[n16]
    LoadToADirectly(u16),
    // LD r8,r8
    // LD r8,n8
    // LD r8,[HL]
    LoadToR8(R8, ByteTarget),
    // LD r16,n16
    LoadToR16(R16, u16),
    // LD SP,n16
    LoadToSP(u16),
    // LD SP,HL
    LoadHLToSP,
    // LD HL,SP+e8
    LoadSPToHL(i8),
    // LD [r16],A
    // LD [HLI],A
    // LD [HLD],A
    StoreA(R16Mem),
    // LD [n16],A
    StoreADirectly(u16),
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
            LD::LoadToADirectly(address) => cpu.registers.a = cpu.bus.read_byte(*address),
            LD::LoadToR8(register, target) => {
                let value = cpu.read_bytetarget(target);
                cpu.write_r8(register, value);
            }
            LD::LoadToR16(register, value) => {
                cpu.write_r16(register, *value);
            }
            LD::LoadToSP(value) => cpu.registers.sp = *value,
            LD::LoadHLToSP => cpu.registers.sp = cpu.read_hl(),
            LD::LoadSPToHL(offset) => {
                let (sp, did_overflow) = cpu.registers.sp.overflowing_add_signed((*offset).into());

                cpu.registers.f.carry = did_overflow;
                let sp_i8 = ((cpu.registers.sp & 0xFF00) >> 8) as i8;
                cpu.registers.f.half_carry = half_carry_set_i8(sp_i8, *offset);

                cpu.write_hl(sp);
            }
            LD::StoreA(target) => cpu.store_at(target, cpu.registers.a),
            LD::StoreADirectly(address) => cpu.bus.write_byte(*address, cpu.registers.a),
            LD::StoreHLRegister(register) => cpu.write_hl_ptr(cpu.read_r8(register)),
            LD::StoreHLConstant(value) => cpu.write_hl_ptr(*value),
            LD::StoreSP(value) => {
                let sp = cpu.registers.sp;
                cpu.bus.write_byte(*value, (sp & 0xFF) as u8);
                cpu.bus.write_byte(*value + 1, (sp >> 8) as u8)
            }
        };
    }
}

pub(crate) enum LDH {
    // LDH A,[n8]
    LoadConstant(u8),
    // LDH A,[C]
    LoadOffset,
    // LDH [n8],A
    StoreConstant(u8),
    // LDH [C],A
    StoreOffset,
}

impl Executable for LDH {
    fn execute(&self, cpu: &mut CPU) {
        match self {
            LDH::LoadConstant(offset) => cpu.registers.a = cpu.bus.read_byte_at_offset(*offset),
            LDH::LoadOffset => cpu.registers.a = cpu.bus.read_byte_at_offset(cpu.registers.c),
            LDH::StoreConstant(offset) => cpu.bus.write_byte_at_offset(*offset, cpu.registers.a),
            LDH::StoreOffset => cpu
                .bus
                .write_byte_at_offset(cpu.registers.c, cpu.registers.a),
        }
    }
}

fn half_carry_set_u8(a: u8, b: u8) -> bool {
    (((a & 0xF) + (b & 0xF)) & 0x10) == 0x10
}

fn half_carry_set_i8(a: i8, b: i8) -> bool {
    (((a & 0xF) + (b & 0xF)) & 0x10) == 0x10
}

fn half_carry_set_u16(a: u16, b: u16) -> bool {
    (((a & 0xFFF) + (b & 0xFFF)) & 0x1000) == 0x1000
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
