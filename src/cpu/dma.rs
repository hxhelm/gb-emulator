use crate::memory::bus::{Bus, OAM_START};

#[derive(Clone, Copy)]
pub(crate) enum DmaState {
    Active { src: u16, index: u16 },
    Inactive,
}

impl DmaState {
    pub(super) fn init_dma_transfer(byte: u8) -> Self {
        DmaState::Active {
            src: (byte as u16) << 8,
            index: 0,
        }
    }

    pub(super) fn step(self, bus: &mut Bus) -> Self {
        match self {
            DmaState::Inactive => DmaState::Inactive,
            DmaState::Active { src, index } => {
                let byte = bus.read_byte(src + index);
                bus.write_byte(OAM_START + index, byte);

                if index == 159 {
                    DmaState::Inactive
                } else {
                    DmaState::Active {
                        src: src,
                        index: index + 1,
                    }
                }
            }
        }
    }
}
