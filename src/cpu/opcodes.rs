use super::{
    instructions::*,
    memory::InstructionData,
    registers::{R16Mem, R8},
};

#[rustfmt::skip]
pub(crate) fn get_instruction(data: &InstructionData) -> Instruction {
    let InstructionData { opcode, param1, param2 } = data;

    match opcode {
        0x06 => Instruction::Ld(LD::LoadToR8(R8::B, ByteTarget::Constant(*param1))),
        0x0E => Instruction::Ld(LD::LoadToR8(R8::C, ByteTarget::Constant(*param1))),
        0x16 => Instruction::Ld(LD::LoadToR8(R8::D, ByteTarget::Constant(*param1))),
        0x1E => Instruction::Ld(LD::LoadToR8(R8::E, ByteTarget::Constant(*param1))),
        0x26 => Instruction::Ld(LD::LoadToR8(R8::H, ByteTarget::Constant(*param1))),
        0x2E => Instruction::Ld(LD::LoadToR8(R8::L, ByteTarget::Constant(*param1))),
        0x3E => Instruction::Ld(LD::LoadToR8(R8::A, ByteTarget::Constant(*param1))),
        0x0A => Instruction::Ld(LD::LoadToA(R16Mem::BC)),
        0x1A => Instruction::Ld(LD::LoadToA(R16Mem::DE)),
        0x2A => Instruction::Ld(LD::LoadToA(R16Mem::HLI)),
        0x3A => Instruction::Ld(LD::LoadToA(R16Mem::HLD)),
        _ => Instruction::Invalid(*opcode),
    }
}
