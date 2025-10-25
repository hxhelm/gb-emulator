use crate::memory::bus::Bus;

use super::memory::OBJECT_SIZE;

#[allow(unused)]
#[derive(Clone, Copy, Default, Debug)]
pub(super) struct ObjectAttribute {
    pub(super) y_position: u8,
    pub(super) x_position: u8,
    pub(super) tile_index: u8,
    pub(super) attributes: u8,
}

pub(super) enum ObjectPriority {
    LOW,
    HIGH,
}

pub(super) enum DmgPalette {
    OBP0,
    OBP1,
}

#[allow(unused)]
impl ObjectAttribute {
    pub(super) fn get_priority(&self) -> ObjectPriority {
        if self.attributes & 0b10000000 == 0 {
            ObjectPriority::HIGH
        } else {
            ObjectPriority::LOW
        }
    }

    pub(super) fn is_y_flipped(&self) -> bool {
        self.attributes & 0b01000000 != 0
    }

    pub(super) fn is_x_flipped(&self) -> bool {
        self.attributes & 0b00100000 != 0
    }

    pub(super) fn get_dmg_palette(&self) -> DmgPalette {
        if self.attributes & 0b00010000 == 0 {
            DmgPalette::OBP0
        } else {
            DmgPalette::OBP1
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct ObjectBuffer {
    pub(super) buffer: [Option<ObjectAttribute>; 10],
    length: usize,
    head: usize,
    tail: usize,
    oam_index: u8,
    t_cycles_elapsed: u16,
}

impl ObjectBuffer {
    pub(super) fn init() -> Self {
        Self {
            buffer: [None; 10],
            length: 0,
            head: 0,
            tail: 0,
            oam_index: 0,
            t_cycles_elapsed: 0,
        }
    }

    pub(super) fn reset_line(&mut self) {
        self.buffer.iter_mut().for_each(|item| *item = None);
        self.length = 0;
        self.oam_index = 0;
        self.t_cycles_elapsed = 0;
    }

    // For the dmg_acid "Hello World" text, the y and x positions are correct and in the right
    // order, the buffer is correctly filled with the 10 characters.
    // The tile index simply points to the ascii representation of a character, with "H" and "W"
    // being uppercase.
    //
    // [src/graphics/ppu.rs:148:21] self.object_buffer.buffer = [
    //     Some(
    //         ObjectAttribute {
    //             y_position: 16,
    //             x_position: 40,
    //             tile_index: 72,
    //             attributes: 0,
    //         },
    //     ),
    //     Some(
    //         ObjectAttribute {
    //             y_position: 16,
    //             x_position: 48,
    //             tile_index: 101,
    //             attributes: 0,
    //         },
    //     ),
    //     Some(
    //         ObjectAttribute {
    //             y_position: 16,
    //             x_position: 56,
    //             tile_index: 108,
    //             attributes: 0,
    //         },
    //     ),
    //     Some(
    //         ObjectAttribute {
    //             y_position: 16,
    //             x_position: 64,
    //             tile_index: 108,
    //             attributes: 0,
    //         },
    //     ),
    //     Some(
    //         ObjectAttribute {
    //             y_position: 16,
    //             x_position: 72,
    //             tile_index: 111,
    //             attributes: 0,
    //         },
    //     ),
    //     Some(
    //         ObjectAttribute {
    //             y_position: 16,
    //             x_position: 88,
    //             tile_index: 87,
    //             attributes: 0,
    //         },
    //     ),
    //     Some(
    //         ObjectAttribute {
    //             y_position: 16,
    //             x_position: 96,
    //             tile_index: 111,
    //             attributes: 0,
    //         },
    //     ),
    //     Some(
    //         ObjectAttribute {
    //             y_position: 16,
    //             x_position: 104,
    //             tile_index: 114,
    //             attributes: 0,
    //         },
    //     ),
    //     Some(
    //         ObjectAttribute {
    //             y_position: 16,
    //             x_position: 112,
    //             tile_index: 108,
    //             attributes: 0,
    //         },
    //     ),
    //     Some(
    //         ObjectAttribute {
    //             y_position: 16,
    //             x_position: 120,
    //             tile_index: 100,
    //             attributes: 0,
    //         },
    //     ),
    // ]
    pub(super) fn step(&mut self, bus: &Bus, passed_t_cycles: u8) {
        if self.is_full() {
            return;
        }

        let current_line = bus.current_line();
        let object_size = bus.get_obj_size();

        self.t_cycles_elapsed += passed_t_cycles as u16;

        while self.t_cycles_elapsed >= 2 && self.oam_index < 40 {
            self.t_cycles_elapsed -= 2;

            let address = (self.oam_index * 4) as u16;
            let object_y = bus.oam.read(address);

            let object_attribute = ObjectAttribute {
                y_position: object_y,
                x_position: bus.oam.read(address + 1),
                tile_index: bus.oam.read(address + 2),
                attributes: bus.oam.read(address + 3),
            };

            if current_line + OBJECT_SIZE < object_y
                || current_line + OBJECT_SIZE >= object_y + object_size
            {
                self.oam_index += 1;
                continue;
            }

            if self.push_back(object_attribute).is_ok() {
                self.oam_index += 1;
            }
        }
    }

    pub fn push_back(&mut self, obj: ObjectAttribute) -> Result<(), ()> {
        if self.is_full() {
            return Err(());
        }

        self.buffer[self.tail] = Some(obj);
        self.tail = (self.tail + 1) % self.buffer.len();
        self.length += 1;
        Ok(())
    }

    pub fn pop_front_if<F>(&mut self, mut condition: F) -> Option<ObjectAttribute>
    where
        F: FnMut(&ObjectAttribute) -> bool,
    {
        if self.length == 0 {
            return None;
        }

        let head_obj = self.buffer[self.head].as_ref()?;

        if condition(head_obj) {
            let val = self.buffer[self.head].take();
            self.head = (self.head + 1) % self.buffer.len();
            self.length -= 1;
            val
        } else {
            None
        }
    }

    fn is_full(&self) -> bool {
        self.length == self.buffer.len()
    }
}
