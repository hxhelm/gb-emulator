use super::{
    instructions::*,
    memory::InstructionData,
    registers::{R16Mem, R16, R8},
};

/// Maps the opcode to a Instruction to execute and returns a tuple consisting of the instruction
/// and the number of bytes used for the instruction.
#[rustfmt::skip]
pub(crate) fn get_instruction(data: &InstructionData) -> (Instruction, u16) {
    let InstructionData { opcode, param1, param2 } = data;

    let get_a16 = || (*param1 as u16) << 8 | (*param2 as u16);

    match opcode {
        0x06 => (Instruction::Ld(LD::LoadToR8(R8::B, ByteTarget::Constant(*param1))), 2),
        0x0E => (Instruction::Ld(LD::LoadToR8(R8::C, ByteTarget::Constant(*param1))), 2),
        0x16 => (Instruction::Ld(LD::LoadToR8(R8::D, ByteTarget::Constant(*param1))), 2),
        0x1E => (Instruction::Ld(LD::LoadToR8(R8::E, ByteTarget::Constant(*param1))), 2),
        0x26 => (Instruction::Ld(LD::LoadToR8(R8::H, ByteTarget::Constant(*param1))), 2),
        0x2E => (Instruction::Ld(LD::LoadToR8(R8::L, ByteTarget::Constant(*param1))), 2),
        0x3E => (Instruction::Ld(LD::LoadToR8(R8::A, ByteTarget::Constant(*param1))), 2),
        0x36 => (Instruction::Ld(LD::StoreHLConstant(*param1)), 2),
        0x40 => (Instruction::Ld(LD::LoadToR8(R8::B, ByteTarget::Register8(R8::B))), 1),
        0x41 => (Instruction::Ld(LD::LoadToR8(R8::B, ByteTarget::Register8(R8::C))), 1),
        0x42 => (Instruction::Ld(LD::LoadToR8(R8::B, ByteTarget::Register8(R8::D))), 1),
        0x43 => (Instruction::Ld(LD::LoadToR8(R8::B, ByteTarget::Register8(R8::E))), 1),
        0x44 => (Instruction::Ld(LD::LoadToR8(R8::B, ByteTarget::Register8(R8::H))), 1),
        0x45 => (Instruction::Ld(LD::LoadToR8(R8::B, ByteTarget::Register8(R8::L))), 1),
        0x46 => (Instruction::Ld(LD::LoadToR8(R8::B, ByteTarget::HLAddress)), 1),
        0x47 => (Instruction::Ld(LD::LoadToR8(R8::B, ByteTarget::Register8(R8::A))), 1),
        0x48 => (Instruction::Ld(LD::LoadToR8(R8::C, ByteTarget::Register8(R8::B))), 1),
        0x49 => (Instruction::Ld(LD::LoadToR8(R8::C, ByteTarget::Register8(R8::C))), 1),
        0x4A => (Instruction::Ld(LD::LoadToR8(R8::C, ByteTarget::Register8(R8::D))), 1),
        0x4B => (Instruction::Ld(LD::LoadToR8(R8::C, ByteTarget::Register8(R8::E))), 1),
        0x4C => (Instruction::Ld(LD::LoadToR8(R8::C, ByteTarget::Register8(R8::H))), 1),
        0x4D => (Instruction::Ld(LD::LoadToR8(R8::C, ByteTarget::Register8(R8::L))), 1),
        0x4E => (Instruction::Ld(LD::LoadToR8(R8::C, ByteTarget::HLAddress)), 1),
        0x4F => (Instruction::Ld(LD::LoadToR8(R8::C, ByteTarget::Register8(R8::A))), 1),
        0x50 => (Instruction::Ld(LD::LoadToR8(R8::D, ByteTarget::Register8(R8::B))), 1),
        0x51 => (Instruction::Ld(LD::LoadToR8(R8::D, ByteTarget::Register8(R8::C))), 1),
        0x52 => (Instruction::Ld(LD::LoadToR8(R8::D, ByteTarget::Register8(R8::D))), 1),
        0x53 => (Instruction::Ld(LD::LoadToR8(R8::D, ByteTarget::Register8(R8::E))), 1),
        0x54 => (Instruction::Ld(LD::LoadToR8(R8::D, ByteTarget::Register8(R8::H))), 1),
        0x55 => (Instruction::Ld(LD::LoadToR8(R8::D, ByteTarget::Register8(R8::L))), 1),
        0x56 => (Instruction::Ld(LD::LoadToR8(R8::D, ByteTarget::HLAddress)), 1),
        0x57 => (Instruction::Ld(LD::LoadToR8(R8::D, ByteTarget::Register8(R8::A))), 1),
        0x58 => (Instruction::Ld(LD::LoadToR8(R8::E, ByteTarget::Register8(R8::B))), 1),
        0x59 => (Instruction::Ld(LD::LoadToR8(R8::E, ByteTarget::Register8(R8::C))), 1),
        0x5A => (Instruction::Ld(LD::LoadToR8(R8::E, ByteTarget::Register8(R8::D))), 1),
        0x5B => (Instruction::Ld(LD::LoadToR8(R8::E, ByteTarget::Register8(R8::E))), 1),
        0x5C => (Instruction::Ld(LD::LoadToR8(R8::E, ByteTarget::Register8(R8::H))), 1),
        0x5D => (Instruction::Ld(LD::LoadToR8(R8::E, ByteTarget::Register8(R8::L))), 1),
        0x5E => (Instruction::Ld(LD::LoadToR8(R8::E, ByteTarget::HLAddress)), 1),
        0x5F => (Instruction::Ld(LD::LoadToR8(R8::E, ByteTarget::Register8(R8::A))), 1),
        0x60 => (Instruction::Ld(LD::LoadToR8(R8::H, ByteTarget::Register8(R8::B))), 1),
        0x61 => (Instruction::Ld(LD::LoadToR8(R8::H, ByteTarget::Register8(R8::C))), 1),
        0x62 => (Instruction::Ld(LD::LoadToR8(R8::H, ByteTarget::Register8(R8::D))), 1),
        0x63 => (Instruction::Ld(LD::LoadToR8(R8::H, ByteTarget::Register8(R8::E))), 1),
        0x64 => (Instruction::Ld(LD::LoadToR8(R8::H, ByteTarget::Register8(R8::H))), 1),
        0x65 => (Instruction::Ld(LD::LoadToR8(R8::H, ByteTarget::Register8(R8::L))), 1),
        0x66 => (Instruction::Ld(LD::LoadToR8(R8::H, ByteTarget::HLAddress)), 1),
        0x67 => (Instruction::Ld(LD::LoadToR8(R8::H, ByteTarget::Register8(R8::A))), 1),
        0x68 => (Instruction::Ld(LD::LoadToR8(R8::L, ByteTarget::Register8(R8::B))), 1),
        0x69 => (Instruction::Ld(LD::LoadToR8(R8::L, ByteTarget::Register8(R8::C))), 1),
        0x6A => (Instruction::Ld(LD::LoadToR8(R8::L, ByteTarget::Register8(R8::D))), 1),
        0x6B => (Instruction::Ld(LD::LoadToR8(R8::L, ByteTarget::Register8(R8::E))), 1),
        0x6C => (Instruction::Ld(LD::LoadToR8(R8::L, ByteTarget::Register8(R8::H))), 1),
        0x6D => (Instruction::Ld(LD::LoadToR8(R8::L, ByteTarget::Register8(R8::L))), 1),
        0x6E => (Instruction::Ld(LD::LoadToR8(R8::L, ByteTarget::HLAddress)), 1),
        0x6F => (Instruction::Ld(LD::LoadToR8(R8::L, ByteTarget::Register8(R8::A))), 1),
        0x70 => (Instruction::Ld(LD::StoreHLRegister(R8::B)), 1),
        0x71 => (Instruction::Ld(LD::StoreHLRegister(R8::C)), 1),
        0x72 => (Instruction::Ld(LD::StoreHLRegister(R8::D)), 1),
        0x73 => (Instruction::Ld(LD::StoreHLRegister(R8::E)), 1),
        0x74 => (Instruction::Ld(LD::StoreHLRegister(R8::H)), 1),
        0x75 => (Instruction::Ld(LD::StoreHLRegister(R8::L)), 1),
        0x77 => (Instruction::Ld(LD::StoreHLRegister(R8::A)), 1),
        0x78 => (Instruction::Ld(LD::LoadToR8(R8::A, ByteTarget::Register8(R8::B))), 1),
        0x79 => (Instruction::Ld(LD::LoadToR8(R8::A, ByteTarget::Register8(R8::C))), 1),
        0x7A => (Instruction::Ld(LD::LoadToR8(R8::A, ByteTarget::Register8(R8::D))), 1),
        0x7B => (Instruction::Ld(LD::LoadToR8(R8::A, ByteTarget::Register8(R8::E))), 1),
        0x7C => (Instruction::Ld(LD::LoadToR8(R8::A, ByteTarget::Register8(R8::H))), 1),
        0x7D => (Instruction::Ld(LD::LoadToR8(R8::A, ByteTarget::Register8(R8::L))), 1),
        0x7E => (Instruction::Ld(LD::LoadToR8(R8::A, ByteTarget::HLAddress)), 1),
        0x7F => (Instruction::Ld(LD::LoadToR8(R8::A, ByteTarget::Register8(R8::A))), 1),
        0x02 => (Instruction::Ld(LD::StoreA(R16Mem::BC)), 1),
        0x12 => (Instruction::Ld(LD::StoreA(R16Mem::DE)), 1),
        0x22 => (Instruction::Ld(LD::StoreA(R16Mem::HLI)), 1),
        0x32 => (Instruction::Ld(LD::StoreA(R16Mem::HLD)), 1),
        0x0A => (Instruction::Ld(LD::LoadToA(R16Mem::BC)), 1),
        0x1A => (Instruction::Ld(LD::LoadToA(R16Mem::DE)), 1),
        0x2A => (Instruction::Ld(LD::LoadToA(R16Mem::HLI)), 1),
        0x3A => (Instruction::Ld(LD::LoadToA(R16Mem::HLD)), 1),
        0x01 => (Instruction::Ld(LD::LoadToR16(R16::BC, get_a16())), 3),
        0x11 => (Instruction::Ld(LD::LoadToR16(R16::DE, get_a16())), 3),
        0x21 => (Instruction::Ld(LD::LoadToR16(R16::HL, get_a16())), 3),
        0x31 => (Instruction::Ld(LD::LoadToSP(get_a16())), 3),
        0x08 => (Instruction::Ld(LD::StoreSP(get_a16())), 3),
        0xF8 => (Instruction::Ld(LD::LoadSPToHL((*param1) as i8)), 2),
        0xF9 => (Instruction::Ld(LD::LoadHLToSP), 1),
        0xEA => (Instruction::Ld(LD::StoreADirectly(get_a16())), 3),
        0xFA => (Instruction::Ld(LD::LoadToADirectly(get_a16())), 3),
        0xE0 => (Instruction::Ldh(LDH::StoreConstant(*param1)), 2),
        0xF0 => (Instruction::Ldh(LDH::LoadConstant(*param1)), 2),
        0xE2 => (Instruction::Ldh(LDH::StoreOffset), 1),
        0xF2 => (Instruction::Ldh(LDH::LoadOffset), 1),
        0x80 => (Instruction::Add(ADD::Byte(ByteTarget::Register8(R8::B))), 1),
        0x81 => (Instruction::Add(ADD::Byte(ByteTarget::Register8(R8::C))), 1),
        0x82 => (Instruction::Add(ADD::Byte(ByteTarget::Register8(R8::D))), 1),
        0x83 => (Instruction::Add(ADD::Byte(ByteTarget::Register8(R8::E))), 1),
        0x84 => (Instruction::Add(ADD::Byte(ByteTarget::Register8(R8::H))), 1),
        0x85 => (Instruction::Add(ADD::Byte(ByteTarget::Register8(R8::L))), 1),
        0x86 => (Instruction::Add(ADD::Byte(ByteTarget::HLAddress)), 1),
        0x87 => (Instruction::Add(ADD::Byte(ByteTarget::Register8(R8::A))), 1),
        0xC6 => (Instruction::Add(ADD::Byte(ByteTarget::Constant(*param1))), 2),
        0x09 => (Instruction::Add(ADD::Word(WordTarget::Register16(R16::BC))), 1),
        0x19 => (Instruction::Add(ADD::Word(WordTarget::Register16(R16::DE))), 1),
        0x29 => (Instruction::Add(ADD::Word(WordTarget::Register16(R16::HL))), 1),
        0x39 => (Instruction::Add(ADD::Word(WordTarget::SP)), 1),
        0xE8 => (Instruction::Add(ADD::StackPointer((*param1) as i8)), 2),
        0x88 => (Instruction::Adc(ADC(ByteTarget::Register8(R8::B))), 1),
        0x89 => (Instruction::Adc(ADC(ByteTarget::Register8(R8::C))), 1),
        0x8a => (Instruction::Adc(ADC(ByteTarget::Register8(R8::D))), 1),
        0x8b => (Instruction::Adc(ADC(ByteTarget::Register8(R8::E))), 1),
        0x8c => (Instruction::Adc(ADC(ByteTarget::Register8(R8::H))), 1),
        0x8d => (Instruction::Adc(ADC(ByteTarget::Register8(R8::L))), 1),
        0x8e => (Instruction::Adc(ADC(ByteTarget::HLAddress)), 1),
        0x8f => (Instruction::Adc(ADC(ByteTarget::Register8(R8::A))), 1),
        0xCE => (Instruction::Adc(ADC(ByteTarget::Constant(*param1))), 2),
        0x90 => (Instruction::Sub(SUB(ByteTarget::Register8(R8::B))), 1),
        0x91 => (Instruction::Sub(SUB(ByteTarget::Register8(R8::C))), 1),
        0x92 => (Instruction::Sub(SUB(ByteTarget::Register8(R8::D))), 1),
        0x93 => (Instruction::Sub(SUB(ByteTarget::Register8(R8::E))), 1),
        0x94 => (Instruction::Sub(SUB(ByteTarget::Register8(R8::H))), 1),
        0x95 => (Instruction::Sub(SUB(ByteTarget::Register8(R8::L))), 1),
        0x96 => (Instruction::Sub(SUB(ByteTarget::HLAddress)), 1),
        0x97 => (Instruction::Sub(SUB(ByteTarget::Register8(R8::A))), 1),
        0xD6 => (Instruction::Sub(SUB(ByteTarget::Constant(*param1))), 2),
        0x98 => (Instruction::Sbc(SBC(ByteTarget::Register8(R8::B))), 1),
        0x99 => (Instruction::Sbc(SBC(ByteTarget::Register8(R8::C))), 1),
        0x9a => (Instruction::Sbc(SBC(ByteTarget::Register8(R8::D))), 1),
        0x9b => (Instruction::Sbc(SBC(ByteTarget::Register8(R8::E))), 1),
        0x9c => (Instruction::Sbc(SBC(ByteTarget::Register8(R8::H))), 1),
        0x9d => (Instruction::Sbc(SBC(ByteTarget::Register8(R8::L))), 1),
        0x9e => (Instruction::Sbc(SBC(ByteTarget::HLAddress)), 1),
        0x9f => (Instruction::Sbc(SBC(ByteTarget::Register8(R8::A))), 1),
        0xDE => (Instruction::Sbc(SBC(ByteTarget::Constant(*param1))), 2),
        0xA0 => (Instruction::And(AND(ByteTarget::Register8(R8::B))), 1),
        0xA1 => (Instruction::And(AND(ByteTarget::Register8(R8::C))), 1),
        0xA2 => (Instruction::And(AND(ByteTarget::Register8(R8::D))), 1),
        0xA3 => (Instruction::And(AND(ByteTarget::Register8(R8::E))), 1),
        0xA4 => (Instruction::And(AND(ByteTarget::Register8(R8::H))), 1),
        0xA5 => (Instruction::And(AND(ByteTarget::Register8(R8::L))), 1),
        0xA6 => (Instruction::And(AND(ByteTarget::HLAddress)), 1),
        0xA7 => (Instruction::And(AND(ByteTarget::Register8(R8::A))), 1),
        0xE6 => (Instruction::And(AND(ByteTarget::Constant(*param1))), 2),
        0xA8 => (Instruction::Xor(XOR(ByteTarget::Register8(R8::B))), 1),
        0xA9 => (Instruction::Xor(XOR(ByteTarget::Register8(R8::C))), 1),
        0xAa => (Instruction::Xor(XOR(ByteTarget::Register8(R8::D))), 1),
        0xAb => (Instruction::Xor(XOR(ByteTarget::Register8(R8::E))), 1),
        0xAc => (Instruction::Xor(XOR(ByteTarget::Register8(R8::H))), 1),
        0xAd => (Instruction::Xor(XOR(ByteTarget::Register8(R8::L))), 1),
        0xAe => (Instruction::Xor(XOR(ByteTarget::HLAddress)), 1),
        0xAf => (Instruction::Xor(XOR(ByteTarget::Register8(R8::A))), 1),
        0xEE => (Instruction::Xor(XOR(ByteTarget::Constant(*param1))), 2),
        0xB0 => (Instruction::Or(OR(ByteTarget::Register8(R8::B))), 1),
        0xB1 => (Instruction::Or(OR(ByteTarget::Register8(R8::C))), 1),
        0xB2 => (Instruction::Or(OR(ByteTarget::Register8(R8::D))), 1),
        0xB3 => (Instruction::Or(OR(ByteTarget::Register8(R8::E))), 1),
        0xB4 => (Instruction::Or(OR(ByteTarget::Register8(R8::H))), 1),
        0xB5 => (Instruction::Or(OR(ByteTarget::Register8(R8::L))), 1),
        0xB6 => (Instruction::Or(OR(ByteTarget::HLAddress)), 1),
        0xB7 => (Instruction::Or(OR(ByteTarget::Register8(R8::A))), 1),
        0xF6 => (Instruction::Or(OR(ByteTarget::Constant(*param1))), 2),
        0xB8 => (Instruction::Cp(CP(ByteTarget::Register8(R8::B))), 1),
        0xB9 => (Instruction::Cp(CP(ByteTarget::Register8(R8::C))), 1),
        0xBa => (Instruction::Cp(CP(ByteTarget::Register8(R8::D))), 1),
        0xBb => (Instruction::Cp(CP(ByteTarget::Register8(R8::E))), 1),
        0xBc => (Instruction::Cp(CP(ByteTarget::Register8(R8::H))), 1),
        0xBd => (Instruction::Cp(CP(ByteTarget::Register8(R8::L))), 1),
        0xBe => (Instruction::Cp(CP(ByteTarget::HLAddress)), 1),
        0xBf => (Instruction::Cp(CP(ByteTarget::Register8(R8::A))), 1),
        0xFE => (Instruction::Cp(CP(ByteTarget::Constant(*param1))), 2),
        0x03 => (Instruction::Inc(INC::R16(R16::BC)), 1),
        0x13 => (Instruction::Inc(INC::R16(R16::DE)), 1),
        0x23 => (Instruction::Inc(INC::R16(R16::HL)), 1),
        0x33 => (Instruction::Inc(INC::SP), 1),
        0x04 => (Instruction::Inc(INC::R8(R8::B)), 1),
        0x0C => (Instruction::Inc(INC::R8(R8::C)), 1),
        0x14 => (Instruction::Inc(INC::R8(R8::D)), 1),
        0x1C => (Instruction::Inc(INC::R8(R8::E)), 1),
        0x24 => (Instruction::Inc(INC::R8(R8::H)), 1),
        0x2C => (Instruction::Inc(INC::R8(R8::L)), 1),
        0x34 => (Instruction::Inc(INC::HL), 1),
        0x3C => (Instruction::Inc(INC::R8(R8::A)), 1),
        0x0B => (Instruction::Dec(DEC::R16(R16::BC)), 1),
        0x1B => (Instruction::Dec(DEC::R16(R16::DE)), 1),
        0x2B => (Instruction::Dec(DEC::R16(R16::HL)), 1),
        0x3B => (Instruction::Dec(DEC::SP), 1),
        0x05 => (Instruction::Dec(DEC::R8(R8::B)), 1),
        0x0D => (Instruction::Dec(DEC::R8(R8::C)), 1),
        0x15 => (Instruction::Dec(DEC::R8(R8::D)), 1),
        0x1D => (Instruction::Dec(DEC::R8(R8::E)), 1),
        0x25 => (Instruction::Dec(DEC::R8(R8::H)), 1),
        0x2D => (Instruction::Dec(DEC::R8(R8::L)), 1),
        0x35 => (Instruction::Dec(DEC::HL), 1),
        0x3D => (Instruction::Dec(DEC::R8(R8::A)), 1),
        0x07 => (Instruction::Rlca(RLCA), 1),
        0x0F => (Instruction::Rrca(RRCA), 1),
        0x17 => (Instruction::Rla(RLA), 1),
        0x1F => (Instruction::Rra(RRA), 1),
        0x27 => (Instruction::Daa(DAA), 1),
        0x2F => (Instruction::Cpl(CPL), 1),
        0x37 => (Instruction::Scf(SCF), 1),
        0x3F => (Instruction::Ccf(CCF), 1),
        0x00 => (Instruction::Nop(NOP), 1),
        0xF3 => (Instruction::Di(DI), 1),
        0xFB => (Instruction::Ei(EI), 1),
        0xC2 => (Instruction::Jp(JP::ConditionalConstant(FlagCondition::NotZero, get_a16())), 3),
        0xCA => (Instruction::Jp(JP::ConditionalConstant(FlagCondition::Zero, get_a16())), 3),
        0xD2 => (Instruction::Jp(JP::ConditionalConstant(FlagCondition::NotCarry, get_a16())), 3),
        0xDA => (Instruction::Jp(JP::ConditionalConstant(FlagCondition::Carry, get_a16())), 3),
        0xC3 => (Instruction::Jp(JP::Constant(get_a16())), 3),
        0xE9 => (Instruction::Jp(JP::HLAddress), 1),
        0x20 => (Instruction::Jr(JR::ConditionalOffset(FlagCondition::NotZero, *param1)), 2),
        0x28 => (Instruction::Jr(JR::ConditionalOffset(FlagCondition::Zero, *param1)), 2),
        0x30 => (Instruction::Jr(JR::ConditionalOffset(FlagCondition::NotCarry, *param1)), 2),
        0x38 => (Instruction::Jr(JR::ConditionalOffset(FlagCondition::Carry, *param1)), 2),
        0x18 => (Instruction::Jr(JR::Offset(*param1)), 2),
        0xC4 => (Instruction::Call(CALL::ConditionalConstant(FlagCondition::NotZero, get_a16())), 3),
        0xCC => (Instruction::Call(CALL::ConditionalConstant(FlagCondition::Zero, get_a16())), 3),
        0xD4 => (Instruction::Call(CALL::ConditionalConstant(FlagCondition::NotCarry, get_a16())), 3),
        0xDC => (Instruction::Call(CALL::ConditionalConstant(FlagCondition::Carry, get_a16())), 3),
        0xCD => (Instruction::Call(CALL::Constant(get_a16())), 3),
        0xC5 => (Instruction::Push(PUSH::R16(R16::BC)), 1),
        0xD5 => (Instruction::Push(PUSH::R16(R16::DE)), 1),
        0xE5 => (Instruction::Push(PUSH::R16(R16::HL)), 1),
        0xF5 => (Instruction::Push(PUSH::AF), 1),
        0xC1 => (Instruction::Pop(POP::R16(R16::BC)), 1),
        0xD1 => (Instruction::Pop(POP::R16(R16::DE)), 1),
        0xE1 => (Instruction::Pop(POP::R16(R16::HL)), 1),
        0xF1 => (Instruction::Pop(POP::AF), 1),
        _ => (Instruction::Invalid(*opcode), 1),
    }
}
