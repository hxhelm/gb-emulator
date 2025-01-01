#![allow(unused)]

// TODO: implement memory bank switching
#[derive(Clone)]
pub struct Memory {
    memory: Vec<u8>,
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            memory: vec![0; 0x8000],
        }
    }
}

impl Memory {
    pub(super) fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub(super) fn write(&mut self, address: u16, byte: u8) {
        self.memory[address as usize] = byte;
    }
}

#[derive(Clone)]
pub struct Addressible<const S: usize> {
    memory: [u8; S],
}

impl<const S: usize> Default for Addressible<S> {
    fn default() -> Self {
        Self { memory: [0; S] }
    }
}

impl<const S: usize> Addressible<S> {
    pub(super) fn read(&self, address: u16) -> u8 {
        let address: usize = address.into();
        self.memory[address]
    }

    pub(super) fn write(&mut self, address: u16, byte: u8) {
        let address: usize = address.into();
        self.memory[address] = byte;
    }
}
