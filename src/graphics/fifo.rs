#![allow(unused)]
#[derive(Copy, Clone)]
pub struct Fifo {
    queue: [u8; 8],
    index: usize,
    length: usize,
}

impl Fifo {
    pub fn new() -> Self {
        Self {
            queue: [0; 8],
            index: 0,
            length: 0,
        }
    }

    pub fn push(&mut self, value: u8) -> Result<(), &'static str> {
        if self.length == self.queue.len() {
            return Err("Queue is full");
        }

        let insert_pos = (self.index + self.length) % self.queue.len();
        self.queue[insert_pos] = value;
        self.length += 1;

        Ok(())
    }

    pub fn pop(&mut self) -> Result<u8, &'static str> {
        if self.length == 0 {
            return Err("Queue is empty");
        }

        let value = self.queue[self.index];
        self.index = (self.index + 1) % self.queue.len();
        self.length -= 1;

        Ok(value)
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn is_full(&self) -> bool {
        self.length == self.queue.len()
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn clear(&mut self) {
        self.index = 0;
        self.length = 0;
    }
}
