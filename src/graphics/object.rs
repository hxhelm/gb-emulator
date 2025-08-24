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
    buffer: [Option<ObjectAttribute>; 10],
    length: usize,
    oam_index: u8,
    t_cycles_elapsed: u8,
}

impl ObjectBuffer {
    pub(super) fn init() -> Self {
        Self {
            buffer: [None; 10],
            length: 0,
            oam_index: 0,
            t_cycles_elapsed: 0,
        }
    }

    pub(super) fn reset_line(&mut self) {
        self.buffer.iter_mut().for_each(|slot| *slot = None);
        self.length = 0;
        self.oam_index = 0;
        self.t_cycles_elapsed = 0;
    }

    pub(super) fn step(&mut self, bus: &Bus, passed_t_cycles: u8) {
        if self.length >= 10 {
            return;
        }

        let current_line = bus.current_line();
        let object_size = bus.get_obj_size();

        self.t_cycles_elapsed += passed_t_cycles;

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

            if !self.try_add(object_attribute) {
                return;
            }

            self.oam_index += 1;
        }
    }

    fn try_add(&mut self, object_attribute: ObjectAttribute) -> bool {
        if self.length >= self.buffer.len() {
            return false;
        }

        self.buffer[self.length] = Some(object_attribute);
        self.length += 1;
        true
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = &ObjectAttribute> {
        self.buffer[..self.length].iter().filter_map(|s| s.as_ref())
    }
}
