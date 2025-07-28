#![allow(dead_code)]

use crate::cpu::interrupts::InterruptState;
use crate::cpu::registers::*;
use crate::cpu::CPU;

use super::prefixed::*;

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
    Rrca(RRCA),
    Rra(RRA),
    Rlca(RLCA),
    Rla(RLA),
    Daa(DAA),
    Cpl(CPL),
    Scf(SCF),
    Ccf(CCF),
    Ld(LD),
    Ldh(LDH),
    Nop(NOP),
    Halt(HALT),
    Stop(STOP),
    Di(DI),
    Ei(EI),
    Jp(JP),
    Jr(JR),
    Call(CALL),
    Push(PUSH),
    Pop(POP),
    Rst(RST),
    Ret(RET),
    Rlc(RLC),
    Rrc(RRC),
    Rl(RL),
    Rr(RR),
    Sla(SLA),
    Sra(SRA),
    Swap(SWAP),
    Srl(SRL),
    Bit(BIT),
    Res(RES),
    Set(SET),
    Invalid(u8),
}

pub(crate) trait Executable {
    fn execute(&self, cpu: &mut CPU) -> u8;
}

impl Executable for Instruction {
    fn execute(&self, cpu: &mut CPU) -> u8 {
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
            Self::Rrca(instruction) => instruction.execute(cpu),
            Self::Rra(instruction) => instruction.execute(cpu),
            Self::Rlca(instruction) => instruction.execute(cpu),
            Self::Rla(instruction) => instruction.execute(cpu),
            Self::Daa(instruction) => instruction.execute(cpu),
            Self::Cpl(instruction) => instruction.execute(cpu),
            Self::Scf(instruction) => instruction.execute(cpu),
            Self::Ccf(instruction) => instruction.execute(cpu),
            Self::Ld(instruction) => instruction.execute(cpu),
            Self::Ldh(instruction) => instruction.execute(cpu),
            Self::Nop(instruction) => instruction.execute(cpu),
            Self::Halt(instruction) => instruction.execute(cpu),
            Self::Stop(instruction) => instruction.execute(cpu),
            Self::Di(instruction) => instruction.execute(cpu),
            Self::Ei(instruction) => instruction.execute(cpu),
            Self::Jp(instruction) => instruction.execute(cpu),
            Self::Jr(instruction) => instruction.execute(cpu),
            Self::Call(instruction) => instruction.execute(cpu),
            Self::Push(instruction) => instruction.execute(cpu),
            Self::Pop(instruction) => instruction.execute(cpu),
            Self::Rst(instruction) => instruction.execute(cpu),
            Self::Ret(instruction) => instruction.execute(cpu),
            Self::Rlc(instruction) => instruction.execute(cpu),
            Self::Rrc(instruction) => instruction.execute(cpu),
            Self::Rl(instruction) => instruction.execute(cpu),
            Self::Rr(instruction) => instruction.execute(cpu),
            Self::Sla(instruction) => instruction.execute(cpu),
            Self::Sra(instruction) => instruction.execute(cpu),
            Self::Swap(instruction) => instruction.execute(cpu),
            Self::Srl(instruction) => instruction.execute(cpu),
            Self::Bit(instruction) => instruction.execute(cpu),
            Self::Res(instruction) => instruction.execute(cpu),
            Self::Set(instruction) => instruction.execute(cpu),
            Self::Invalid(opcode) => {
                panic!(
                    "Invalid opcode [{}] | 0x{:02X}. PC: {:04X}",
                    opcode, opcode, cpu.registers.pc
                );
            }
        }
    }
}

pub(crate) enum ByteTarget {
    Constant(u8),
    Register8(R8),
    HLAddress,
}

impl ByteTarget {
    fn cycles(&self) -> u8 {
        match self {
            ByteTarget::Register8(_) => 4,
            _ => 8,
        }
    }
}

pub(crate) enum WordTarget {
    Register16(R16),
    SP,
}

pub(crate) enum ADD {
    Byte(ByteTarget),
    Word(WordTarget),
    StackPointer(u8),
}

impl Executable for ADD {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        match self {
            ADD::Byte(target) => {
                let value = cpu.read_bytetarget(target);
                let (result, did_overflow) = cpu.registers.a.overflowing_add(value);

                cpu.registers.f.zero = result == 0;
                cpu.registers.f.negative = false;
                cpu.registers.f.carry = did_overflow;
                cpu.registers.f.half_carry = half_carry_set_add_u8(cpu.registers.a, value);

                cpu.registers.a = result;
                target.cycles()
            }
            ADD::Word(target) => {
                let value = match target {
                    WordTarget::Register16(register) => cpu.read_r16(register),
                    WordTarget::SP => cpu.registers.sp,
                };
                let (result, did_overflow) = cpu.read_hl().overflowing_add(value);

                cpu.registers.f.negative = false;
                cpu.registers.f.carry = did_overflow;
                cpu.registers.f.half_carry = half_carry_set_add_u16(cpu.read_hl(), value);

                cpu.write_hl(result);
                8
            }
            ADD::StackPointer(offset) => {
                let result = cpu.registers.sp.wrapping_add_signed((*offset as i8).into());

                cpu.registers.f.zero = false;
                cpu.registers.f.negative = false;

                let sp_u8 = cpu.registers.sp as u8;
                cpu.registers.f.half_carry = half_carry_set_add_u8(sp_u8, *offset);
                cpu.registers.f.carry = (result as u8) < sp_u8;

                cpu.registers.sp = result;
                16
            }
        }
    }
}

pub(crate) struct ADC(pub(crate) ByteTarget);

impl Executable for ADC {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        let carry_val: u8 = if cpu.registers.f.carry { 1 } else { 0 };
        let value = cpu.read_bytetarget(&self.0);

        let (result, carry_overflow) = cpu.registers.a.overflowing_add(carry_val);
        let (result, did_overflow) = result.overflowing_add(value);

        cpu.registers.f.zero = result == 0;
        cpu.registers.f.negative = false;
        cpu.registers.f.carry = did_overflow | carry_overflow;
        cpu.registers.f.half_carry = half_carry_set_adc_u8(cpu.registers.a, value, carry_val);

        cpu.registers.a = result;
        self.0.cycles()
    }
}

pub(crate) struct SUB(pub(crate) ByteTarget);

impl Executable for SUB {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        let value = cpu.read_bytetarget(&self.0);
        let (result, did_overflow) = cpu.registers.a.overflowing_sub(value);

        cpu.registers.f.zero = result == 0;
        cpu.registers.f.negative = true;
        cpu.registers.f.carry = did_overflow;
        cpu.registers.f.half_carry = half_carry_set_sub_u8(cpu.registers.a, value);

        cpu.registers.a = result;
        self.0.cycles()
    }
}

pub(crate) struct SBC(pub(crate) ByteTarget);

impl Executable for SBC {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        let carry_val: u8 = if cpu.registers.f.carry { 1 } else { 0 };
        let value = cpu.read_bytetarget(&self.0);

        let (result, carry_overflow) = cpu.registers.a.overflowing_sub(carry_val);
        let (result, did_overflow) = result.overflowing_sub(value);

        cpu.registers.f.zero = result == 0;
        cpu.registers.f.negative = true;
        cpu.registers.f.carry = did_overflow | carry_overflow;
        cpu.registers.f.half_carry = half_carry_set_sbc_u8(cpu.registers.a, value, carry_val);

        cpu.registers.a = result;
        self.0.cycles()
    }
}

pub(crate) struct AND(pub(crate) ByteTarget);

impl Executable for AND {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        let value = cpu.read_bytetarget(&self.0);
        let result = cpu.registers.a & value;

        cpu.registers.f.zero = result == 0;
        cpu.registers.f.negative = false;
        cpu.registers.f.carry = false;
        cpu.registers.f.half_carry = true;

        cpu.registers.a = result;
        self.0.cycles()
    }
}

pub(crate) struct XOR(pub(crate) ByteTarget);

impl Executable for XOR {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        let value = cpu.read_bytetarget(&self.0);
        let result = cpu.registers.a ^ value;

        cpu.registers.f.zero = result == 0;
        cpu.registers.f.negative = false;
        cpu.registers.f.carry = false;
        cpu.registers.f.half_carry = false;

        cpu.registers.a = result;
        self.0.cycles()
    }
}

pub(crate) struct OR(pub(crate) ByteTarget);

impl Executable for OR {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        let value = cpu.read_bytetarget(&self.0);
        let result = cpu.registers.a | value;

        cpu.registers.f.zero = result == 0;
        cpu.registers.f.negative = false;
        cpu.registers.f.carry = false;
        cpu.registers.f.half_carry = false;

        cpu.registers.a = result;
        self.0.cycles()
    }
}

pub(crate) struct CP(pub(crate) ByteTarget);

impl Executable for CP {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        let value = cpu.read_bytetarget(&self.0);
        let (result, did_overflow) = cpu.registers.a.overflowing_sub(value);

        cpu.registers.f.zero = result == 0;
        cpu.registers.f.negative = true;
        cpu.registers.f.carry = did_overflow;
        cpu.registers.f.half_carry = half_carry_set_sub_u8(cpu.registers.a, value);
        self.0.cycles()
    }
}

pub(crate) enum INC {
    R8(R8),
    HL,
    R16(R16),
    SP,
}

impl Executable for INC {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        match self {
            INC::R8(register) => {
                let current = cpu.read_r8(register);
                let result = current.wrapping_add(1);

                cpu.registers.f.zero = result == 0;
                cpu.registers.f.negative = false;
                cpu.registers.f.half_carry = half_carry_set_add_u8(current, 1);

                cpu.write_r8(register, result);
                4
            }
            INC::HL => {
                let current = cpu.read_hl_ptr();
                let result = current.wrapping_add(1);

                cpu.registers.f.zero = result == 0;
                cpu.registers.f.negative = false;
                cpu.registers.f.half_carry = half_carry_set_add_u8(current, 1);

                cpu.write_hl_ptr(result);
                12
            }
            INC::R16(register) => {
                let result = cpu.read_r16(register).wrapping_add(1);

                cpu.write_r16(register, result);
                8
            }
            INC::SP => {
                let result = cpu.registers.sp.wrapping_add(1);

                cpu.registers.sp = result;
                8
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
    fn execute(&self, cpu: &mut CPU) -> u8 {
        match self {
            DEC::R8(register) => {
                let current = cpu.read_r8(register);
                let result = current.wrapping_sub(1);

                cpu.registers.f.zero = result == 0;
                cpu.registers.f.negative = true;
                cpu.registers.f.half_carry = half_carry_set_sub_u8(current, 1);

                cpu.write_r8(register, result);
                4
            }
            DEC::HL => {
                let current = cpu.read_hl_ptr();
                let result = current.wrapping_sub(1);

                cpu.registers.f.zero = result == 0;
                cpu.registers.f.negative = true;
                cpu.registers.f.half_carry = half_carry_set_sub_u8(current, 1);

                cpu.write_hl_ptr(result);
                12
            }
            DEC::R16(register) => {
                let result = cpu.read_r16(register).wrapping_sub(1);

                cpu.write_r16(register, result);
                8
            }
            DEC::SP => {
                let result = cpu.registers.sp.wrapping_sub(1);

                cpu.registers.sp = result;
                8
            }
        }
    }
}

pub(crate) struct RRCA;

impl Executable for RRCA {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        let rotated = cpu.registers.a.rotate_right(1);

        cpu.registers.f.zero = false;
        cpu.registers.f.negative = false;
        cpu.registers.f.half_carry = false;
        cpu.registers.f.carry = cpu.registers.a & 0x1 != 0;
        cpu.registers.a = rotated;
        4
    }
}

pub(crate) struct RLCA;

impl Executable for RLCA {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        let rotated = cpu.registers.a.rotate_left(1);

        cpu.registers.f.zero = false;
        cpu.registers.f.negative = false;
        cpu.registers.f.half_carry = false;
        cpu.registers.f.carry = cpu.registers.a & 0x80 != 0;
        cpu.registers.a = rotated;
        4
    }
}

pub(crate) struct RRA;

impl Executable for RRA {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        let rotated = if cpu.registers.f.carry {
            cpu.registers.a.rotate_right(1) | 0x80
        } else {
            cpu.registers.a.rotate_right(1) & !0x80
        };

        cpu.registers.f.zero = false;
        cpu.registers.f.negative = false;
        cpu.registers.f.half_carry = false;
        cpu.registers.f.carry = cpu.registers.a & 0x1 != 0;
        cpu.registers.a = rotated;
        4
    }
}

pub(crate) struct RLA;

impl Executable for RLA {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        let rotated = if cpu.registers.f.carry {
            cpu.registers.a.rotate_left(1) | 0x01
        } else {
            cpu.registers.a.rotate_left(1) & !0x01
        };

        cpu.registers.f.zero = false;
        cpu.registers.f.negative = false;
        cpu.registers.f.half_carry = false;
        cpu.registers.f.carry = cpu.registers.a & 0x80 != 0;
        cpu.registers.a = rotated;
        4
    }
}

pub(crate) struct DAA;

// Special thanks to https://blog.ollien.com/posts/gb-daa/
impl Executable for DAA {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        let a = cpu.registers.a;
        let mut correction = 0;

        if cpu.registers.f.half_carry || (!cpu.registers.f.negative && (a & 0x0F) > 0x09) {
            correction |= 0x06;
        }

        if cpu.registers.f.carry || (!cpu.registers.f.negative && a > 0x99) {
            correction |= 0x60;
        }

        let new_a = if cpu.registers.f.negative {
            a.wrapping_sub(correction)
        } else {
            a.wrapping_add(correction)
        };

        cpu.registers.f.zero = new_a == 0;
        cpu.registers.f.half_carry = false;
        cpu.registers.f.carry = correction >= 0x60;
        cpu.registers.a = new_a;
        4
    }
}

pub(crate) struct CPL;

impl Executable for CPL {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        cpu.registers.f.negative = true;
        cpu.registers.f.half_carry = true;
        cpu.registers.a = !cpu.registers.a;
        4
    }
}

pub(crate) struct SCF;

impl Executable for SCF {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        cpu.registers.f.negative = false;
        cpu.registers.f.half_carry = false;
        cpu.registers.f.carry = true;
        4
    }
}

pub(crate) struct CCF;

impl Executable for CCF {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        cpu.registers.f.negative = false;
        cpu.registers.f.half_carry = false;
        cpu.registers.f.carry = !cpu.registers.f.carry;
        4
    }
}

pub(crate) enum FlagCondition {
    Zero,
    NotZero,
    Carry,
    NotCarry,
}

impl FlagCondition {
    fn is_true(&self, cpu: &CPU) -> bool {
        match self {
            Self::Zero => cpu.registers.f.zero,
            Self::NotZero => !cpu.registers.f.zero,
            Self::Carry => cpu.registers.f.carry,
            Self::NotCarry => !cpu.registers.f.carry,
        }
    }
}

pub(crate) enum JP {
    Constant(u16),
    ConditionalConstant(FlagCondition, u16),
    HLAddress,
}

impl Executable for JP {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        match self {
            JP::Constant(address) => {
                cpu.registers.pc = *address;
                16
            }
            JP::ConditionalConstant(condition, address) => {
                if condition.is_true(cpu) {
                    cpu.registers.pc = *address;
                    16
                } else {
                    12
                }
            }
            JP::HLAddress => {
                cpu.registers.pc = cpu.read_hl();
                4
            }
        }
    }
}

pub(crate) enum JR {
    Offset(u8),
    ConditionalOffset(FlagCondition, u8),
}

impl Executable for JR {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        match self {
            JR::Offset(address) => {
                cpu.registers.pc = cpu
                    .registers
                    .pc
                    .wrapping_add_signed(i16::from(*address as i8));
                12
            }
            JR::ConditionalOffset(condition, address) => {
                if condition.is_true(cpu) {
                    cpu.registers.pc = cpu
                        .registers
                        .pc
                        .wrapping_add_signed(i16::from(*address as i8));
                    12
                } else {
                    8
                }
            }
        }
    }
}

pub(crate) enum CALL {
    Constant(u16),
    ConditionalConstant(FlagCondition, u16),
}

impl Executable for CALL {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        match self {
            CALL::Constant(address) => {
                cpu.call_address(*address);
                24
            }
            CALL::ConditionalConstant(condition, address) => {
                if condition.is_true(cpu) {
                    cpu.call_address(*address);
                    24
                } else {
                    12
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) enum RST {
    V0 = 0x00,
    V8 = 0x08,
    V10 = 0x10,
    V18 = 0x18,
    V20 = 0x20,
    V28 = 0x28,
    V30 = 0x30,
    V38 = 0x38,
}

impl Executable for RST {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        cpu.call_address(*self as u16);

        16
    }
}

pub(crate) enum RET {
    RET,
    Conditional(FlagCondition),
    EI,
}

impl Executable for RET {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        match self {
            RET::RET => {
                cpu.registers.pc = cpu.pop_from_stack();
                16
            }
            RET::Conditional(condition) => {
                if condition.is_true(cpu) {
                    cpu.registers.pc = cpu.pop_from_stack();
                    20
                } else {
                    8
                }
            }
            RET::EI => {
                cpu.registers.pc = cpu.pop_from_stack();
                cpu.interrupt_state = InterruptState::Enabled;
                16
            }
        }
    }
}

pub(crate) enum PUSH {
    AF,
    R16(R16),
}

impl Executable for PUSH {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        match self {
            PUSH::AF => cpu.push_to_stack(cpu.read_af()),
            PUSH::R16(register) => cpu.push_to_stack(cpu.read_r16(register)),
        };
        16
    }
}

pub(crate) enum POP {
    AF,
    R16(R16),
}

impl Executable for POP {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        match self {
            POP::AF => {
                let [a, f] = cpu.pop_from_stack().to_be_bytes();
                cpu.registers.a = a;
                cpu.registers.f = Flags::from(f);
            }
            POP::R16(register) => {
                let value = cpu.pop_from_stack();
                cpu.write_r16(register, value);
            }
        };
        12
    }
}

pub(crate) enum LD {
    LoadToA(R16Mem),
    LoadToADirectly(u16),
    LoadToR8(R8, ByteTarget),
    LoadToR16(R16, u16),
    LoadToSP(u16),
    LoadHLToSP,
    LoadSPToHL(u8),
    StoreA(R16Mem),
    StoreADirectly(u16),
    StoreHLRegister(R8),
    StoreHLConstant(u8),
    StoreSP(u16),
}

impl Executable for LD {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        match self {
            LD::LoadToA(target) => {
                cpu.registers.a = cpu.read_from(target);
                8
            }
            LD::LoadToADirectly(address) => {
                cpu.registers.a = cpu.bus.read_byte(*address);
                16
            }
            LD::LoadToR8(register, target) => {
                let value = cpu.read_bytetarget(target);
                cpu.write_r8(register, value);
                target.cycles()
            }
            LD::LoadToR16(register, value) => {
                cpu.write_r16(register, *value);
                12
            }
            LD::LoadToSP(value) => {
                cpu.registers.sp = *value;
                12
            }
            LD::LoadHLToSP => {
                cpu.registers.sp = cpu.read_hl();
                8
            }
            LD::LoadSPToHL(offset) => {
                let result = cpu.registers.sp.wrapping_add_signed((*offset as i8).into());

                cpu.registers.f.zero = false;
                cpu.registers.f.negative = false;

                let sp_u8 = cpu.registers.sp as u8;
                cpu.registers.f.half_carry = half_carry_set_add_u8(sp_u8, *offset);
                cpu.registers.f.carry = (result as u8) < sp_u8;

                cpu.write_hl(result);
                12
            }
            LD::StoreA(target) => {
                cpu.store_at(target, cpu.registers.a);
                8
            }
            LD::StoreADirectly(address) => {
                cpu.bus.write_byte(*address, cpu.registers.a);
                16
            }
            LD::StoreHLRegister(register) => {
                cpu.write_hl_ptr(cpu.read_r8(register));
                8
            }
            LD::StoreHLConstant(value) => {
                cpu.write_hl_ptr(*value);
                12
            }
            LD::StoreSP(value) => {
                cpu.bus.write_word(*value, cpu.registers.sp);
                20
            }
        }
    }
}

pub(crate) enum LDH {
    LoadConstant(u8),
    LoadOffset,
    StoreConstant(u8),
    StoreOffset,
}

impl Executable for LDH {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        match self {
            LDH::LoadConstant(offset) => {
                cpu.registers.a = cpu.bus.read_byte_at_offset(*offset);
                12
            }
            LDH::LoadOffset => {
                cpu.registers.a = cpu.bus.read_byte_at_offset(cpu.registers.c);
                8
            }
            LDH::StoreConstant(offset) => {
                cpu.bus.write_byte_at_offset(*offset, cpu.registers.a);
                12
            }
            LDH::StoreOffset => {
                cpu.bus
                    .write_byte_at_offset(cpu.registers.c, cpu.registers.a);
                8
            }
        }
    }
}

pub(crate) struct NOP;

impl Executable for NOP {
    fn execute(&self, _cpu: &mut CPU) -> u8 {
        4
    }
}

pub(crate) struct STOP(pub(crate) u8);

impl Executable for STOP {
    fn execute(&self, _cpu: &mut CPU) -> u8 {
        // TODO: implement
        4
    }
}

pub(crate) struct HALT;

impl Executable for HALT {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        cpu.update_halt_state();

        4
    }
}

pub(crate) struct DI;

impl Executable for DI {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        cpu.interrupt_state = InterruptState::Disabled;
        4
    }
}

pub(crate) struct EI;

impl Executable for EI {
    fn execute(&self, cpu: &mut CPU) -> u8 {
        cpu.interrupt_state = InterruptState::EnableRequested;
        4
    }
}

fn half_carry_set_add_u8(a: u8, b: u8) -> bool {
    ((a & 0xF) + (b & 0xF)) & 0x10 == 0x10
}

fn half_carry_set_adc_u8(a: u8, b: u8, carry: u8) -> bool {
    ((a & 0xF) + (b & 0xF) + (carry & 0xF)) & 0x10 == 0x10
}

fn half_carry_set_sub_u8(a: u8, b: u8) -> bool {
    (((a & 0xF).wrapping_sub(b & 0xF)) & 0x10) == 0x10
}

fn half_carry_set_sbc_u8(a: u8, b: u8, carry: u8) -> bool {
    (a & 0xF).wrapping_sub(b & 0xF).wrapping_sub(carry & 0xF) & 0x10 == 0x10
}

fn half_carry_set_add_u16(a: u16, b: u16) -> bool {
    ((a & 0xFFF) + (b & 0xFFF)) & 0x1000 == 0x1000
}

impl CPU {
    fn push_to_stack(&mut self, value: u16) {
        self.registers.sp = self.registers.sp.wrapping_sub(2);
        self.bus.write_word(self.registers.sp, value);
    }

    fn pop_from_stack(&mut self) -> u16 {
        let result = self.bus.read_word(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(2);
        result
    }

    fn read_bytetarget(&self, target: &ByteTarget) -> u8 {
        match target {
            ByteTarget::Constant(value) => *value,
            ByteTarget::Register8(register) => self.read_r8(register),
            ByteTarget::HLAddress => self.read_hl_ptr(),
        }
    }

    pub(crate) fn call_address(&mut self, address: u16) {
        // we simply push the current PC because the PC is incremented in `CPU::step` before
        // instruction execution
        self.push_to_stack(self.registers.pc);
        self.registers.pc = address;
    }
}
