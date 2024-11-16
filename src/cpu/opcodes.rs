use super::{instructions::*, registers::R8};

fn get_instruction_from_byte(byte: u8) -> Option<Instruction> {
    let n1: u8 = 0;

    match byte {
        0x06..0x2E => {
            let register = match byte {
                0x06 => R8::B,
                0x0E => R8::C,
                0x16 => R8::D,
                0x1E => R8::E,
                0x26 => R8::H,
                0x2E => R8::L,
                _ => return None,
            };

            Some(Instruction::Ld(LD::LoadToR8(
                register,
                ByteTarget::Constant(n1),
            )))
        }
        _ => None,
    }
}
