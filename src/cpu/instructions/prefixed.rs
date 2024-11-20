use crate::cpu::registers::*;

use super::Executable;

pub(crate) enum RLC {
    Register8(R8),
    HLAddress,
}

impl Executable for RLC {
    fn execute(&self, cpu: &mut crate::cpu::CPU) {
        match self {
            RLC::Register8(register) => {
                let value = cpu.read_r8(register);
                let rotated = value.rotate_left(1);

                cpu.registers.f.carry = value & 0x80 != 0;
                cpu.registers.f.zero = rotated == 0;
                cpu.write_r8(register, rotated);
            }
            RLC::HLAddress => {
                let value = cpu.read_hl_ptr();
                let rotated = value.rotate_left(1);

                cpu.registers.f.carry = value & 0x80 != 0;
                cpu.registers.f.zero = rotated == 0;
                cpu.write_hl_ptr(rotated);
            }
        }

        cpu.registers.f.negative = false;
        cpu.registers.f.half_carry = false;
    }
}

pub(crate) enum RRC {
    Register8(R8),
    HLAddress,
}

impl Executable for RRC {
    fn execute(&self, cpu: &mut crate::cpu::CPU) {
        match self {
            RRC::Register8(register) => {
                let value = cpu.read_r8(register);
                let rotated = value.rotate_right(1);

                cpu.registers.f.carry = value & 0x1 != 0;
                cpu.registers.f.zero = rotated == 0;
                cpu.write_r8(register, rotated);
            }
            RRC::HLAddress => {
                let value = cpu.read_hl_ptr();
                let rotated = value.rotate_right(1);

                cpu.registers.f.carry = value & 0x1 != 0;
                cpu.registers.f.zero = rotated == 0;
                cpu.write_hl_ptr(rotated);
            }
        }

        cpu.registers.f.negative = false;
        cpu.registers.f.half_carry = false;
    }
}

pub(crate) enum RL {
    Register8(R8),
    HLAddress,
}

impl Executable for RL {
    fn execute(&self, cpu: &mut crate::cpu::CPU) {
        match self {
            RL::Register8(register) => {
                let value = cpu.read_r8(register);

                let rotated = if cpu.registers.f.carry {
                    value.rotate_left(1) | 0x01
                } else {
                    value.rotate_left(1) & !0x01
                };

                cpu.registers.f.carry = value & 0x80 != 0;
                cpu.registers.f.zero = rotated == 0;
                cpu.write_r8(register, rotated);
            }
            RL::HLAddress => {
                let value = cpu.read_hl_ptr();

                let rotated = if cpu.registers.f.carry {
                    value.rotate_left(1) | 0x01
                } else {
                    value.rotate_left(1) & !0x01
                };

                cpu.registers.f.carry = value & 0x80 != 0;
                cpu.registers.f.zero = rotated == 0;
                cpu.write_hl_ptr(rotated);
            }
        }

        cpu.registers.f.negative = false;
        cpu.registers.f.half_carry = false;
    }
}

pub(crate) enum RR {
    Register8(R8),
    HLAddress,
}

impl Executable for RR {
    fn execute(&self, cpu: &mut crate::cpu::CPU) {
        match self {
            RR::Register8(register) => {
                let value = cpu.read_r8(register);

                let rotated = if cpu.registers.f.carry {
                    value.rotate_right(1) | 0x80
                } else {
                    value.rotate_right(1) & !0x80
                };

                cpu.registers.f.carry = value & 0x1 != 0;
                cpu.registers.f.zero = rotated == 0;
                cpu.write_r8(register, rotated);
            }
            RR::HLAddress => {
                let value = cpu.read_hl_ptr();

                let rotated = if cpu.registers.f.carry {
                    value.rotate_right(1) | 0x80
                } else {
                    value.rotate_right(1) & !0x80
                };

                cpu.registers.f.carry = value & 0x1 != 0;
                cpu.registers.f.zero = rotated == 0;
                cpu.write_hl_ptr(rotated);
            }
        }

        cpu.registers.f.negative = false;
        cpu.registers.f.half_carry = false;
    }
}

pub(crate) enum SLA {
    Register8(R8),
    HLAddress,
}

impl Executable for SLA {
    fn execute(&self, cpu: &mut crate::cpu::CPU) {
        match self {
            SLA::Register8(register) => {
                let value = cpu.read_r8(register);

                let shifted = value << 1;

                cpu.registers.f.carry = value & 0x80 != 0;
                cpu.registers.f.zero = shifted == 0;

                cpu.write_r8(register, shifted);
            }
            SLA::HLAddress => {
                let value = cpu.read_hl_ptr();

                let shifted = value << 1;

                cpu.registers.f.carry = value & 0x80 != 0;
                cpu.registers.f.zero = shifted == 0;

                cpu.write_hl_ptr(shifted);
            }
        }

        cpu.registers.f.negative = false;
        cpu.registers.f.half_carry = false;
    }
}

pub(crate) enum SRA {
    Register8(R8),
    HLAddress,
}

impl Executable for SRA {
    fn execute(&self, cpu: &mut crate::cpu::CPU) {
        match self {
            SRA::Register8(register) => {
                let value = cpu.read_r8(register);

                let shifted = ((value as i8) >> 1) as u8;

                cpu.registers.f.carry = value & 0x01 != 0;
                cpu.registers.f.zero = shifted == 0;

                cpu.write_r8(register, shifted);
            }
            SRA::HLAddress => {
                let value = cpu.read_hl_ptr();

                let shifted = ((value as i8) >> 1) as u8;

                cpu.registers.f.carry = value & 0x01 != 0;
                cpu.registers.f.zero = shifted == 0;

                cpu.write_hl_ptr(shifted);
            }
        }

        cpu.registers.f.negative = false;
        cpu.registers.f.half_carry = false;
    }
}

pub(crate) enum SWAP {
    Register8(R8),
    HLAddress,
}

impl Executable for SWAP {
    fn execute(&self, cpu: &mut crate::cpu::CPU) {
        match self {
            SWAP::Register8(register) => {
                let value = cpu.read_r8(register);
                let swapped = value.rotate_right(4);
                cpu.registers.f.negative = swapped == 0;
                cpu.write_r8(register, swapped);
            }
            SWAP::HLAddress => {
                let value = cpu.read_hl_ptr();
                let swapped = value.rotate_right(4);
                cpu.registers.f.negative = swapped == 0;
                cpu.write_hl_ptr(swapped);
            }
        }

        cpu.registers.f.negative = false;
        cpu.registers.f.carry = false;
        cpu.registers.f.half_carry = false;
    }
}

pub(crate) enum SRL {
    Register8(R8),
    HLAddress,
}

impl Executable for SRL {
    fn execute(&self, cpu: &mut crate::cpu::CPU) {
        match self {
            SRL::Register8(register) => {
                let value = cpu.read_r8(register);

                let shifted = value >> 1;

                cpu.registers.f.carry = value & 0x01 != 0;
                cpu.registers.f.zero = shifted == 0;

                cpu.write_r8(register, shifted);
            }
            SRL::HLAddress => {
                let value = cpu.read_hl_ptr();

                let shifted = value >> 1;

                cpu.registers.f.carry = value & 0x01 != 0;
                cpu.registers.f.zero = shifted == 0;

                cpu.write_hl_ptr(shifted);
            }
        }

        cpu.registers.f.negative = false;
        cpu.registers.f.half_carry = false;
    }
}

#[derive(Clone, Copy)]
pub(crate) enum U3 {
    B0 = 0,
    B1 = 1,
    B2 = 2,
    B3 = 3,
    B4 = 4,
    B5 = 5,
    B6 = 6,
    B7 = 7,
}

impl U3 {
    fn as_bit_mask(&self) -> u8 {
        (1 << (*self as u8)) as u8
    }
}

pub(crate) enum BIT {
    Register8(U3, R8),
    HLAddress(U3),
}

impl Executable for BIT {
    fn execute(&self, cpu: &mut crate::cpu::CPU) {
        let (bit, value) = match self {
            BIT::Register8(bit, register) => (bit, cpu.read_r8(register)),
            BIT::HLAddress(bit) => (bit, cpu.read_hl_ptr()),
        };

        cpu.registers.f.zero = bit.as_bit_mask() & value == 0;
        cpu.registers.f.negative = false;
        cpu.registers.f.half_carry = true;
    }
}

pub(crate) enum RES {
    Register8(U3, R8),
    HLAddress(U3),
}

impl Executable for RES {
    fn execute(&self, cpu: &mut crate::cpu::CPU) {
        match self {
            RES::Register8(bit, register) => {
                cpu.write_r8(register, cpu.read_r8(register) & !bit.as_bit_mask())
            }
            RES::HLAddress(bit) => cpu.write_hl_ptr(cpu.read_hl_ptr() & !bit.as_bit_mask()),
        };
    }
}

pub(crate) enum SET {
    Register8(U3, R8),
    HLAddress(U3),
}

impl Executable for SET {
    fn execute(&self, cpu: &mut crate::cpu::CPU) {
        match self {
            SET::Register8(bit, register) => {
                cpu.write_r8(register, cpu.read_r8(register) | bit.as_bit_mask())
            }
            SET::HLAddress(bit) => cpu.write_hl_ptr(cpu.read_hl_ptr() | bit.as_bit_mask()),
        };
    }
}
