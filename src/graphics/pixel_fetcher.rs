#![allow(unused)]
use crate::memory::bus::Bus;

use super::{
    fifo::Fifo,
    object::{ObjectAttribute, ObjectBuffer},
    PixelData, LCD_WIDTH,
};

const SCROLL_X: u16 = 0xFF43;
const SCROLL_Y: u16 = 0xFF42;
const WINDOW_X: u16 = 0xFF4B;
const WINDOW_Y: u16 = 0xFF4A;
const TILE_SIZE: u8 = 8;

#[derive(Clone, Copy)]
enum PixelFetcherStep {
    GetTile,
    GetTileDataLow,
    GetTileDataHigh,
    Sleep(u8),
    Push,
}

#[derive(Default, Clone, Copy)]
enum FetchMode {
    #[default]
    Background,
    Window,
    Object(ObjectAttribute),
}

#[derive(Clone, Copy)]
pub(crate) struct PixelFetcher {
    current_step: PixelFetcherStep,
    current_line: u8,
    /// Tile number fetching
    fetch_mode: FetchMode,
    fetcher_x: u8,
    tile_number: u8,
    window_line_counter: u8,
    /// Temporary tile data
    tile_data_address: u16,
    tile_data_low: u8,
    tile_data_high: u8,
    /// Keeps track of how many pixels need to be discarded at the start of scanline
    discard_counter: u8,
    /// FIFO queues
    background_queue: Fifo,
    sprite_queue: Fifo,
    render_x: u8,
}

impl PixelFetcher {
    pub fn init() -> Self {
        Self {
            current_step: PixelFetcherStep::Sleep(6),
            current_line: 0,
            fetch_mode: FetchMode::default(),
            fetcher_x: 0,
            tile_number: 0,
            window_line_counter: 0,
            tile_data_address: 0,
            tile_data_low: 0,
            tile_data_high: 0,
            discard_counter: 0,
            background_queue: Fifo::new(),
            sprite_queue: Fifo::new(),
            render_x: 0,
        }
    }

    pub fn reset_line(&mut self, bus: &Bus) {
        let window_line_counter = if matches!(self.fetch_mode, FetchMode::Window) {
            self.window_line_counter.wrapping_add(1)
        } else {
            self.window_line_counter
        };

        *self = Self::init();

        self.current_line = bus.current_line();
        self.window_line_counter = window_line_counter;
        self.discard_counter = bus.get_scroll_x() % TILE_SIZE;
    }

    pub fn reset_frame(&mut self, bus: &Bus) {
        *self = Self::init();

        self.current_line = bus.current_line();
        self.discard_counter = bus.get_scroll_x() % TILE_SIZE;
    }

    // TODO: avoid taking and writing the PixelData directly and just return individual pixels
    // instead
    // TODO: return any penalties that extend the number of cycles mode 3 takes
    pub fn step(
        &mut self,
        bus: &Bus,
        object_buffer: &ObjectBuffer,
        passed_t_cycles: u8,
        current_frame: &mut PixelData,
    ) {
        let mut t_counter = 0;

        // TODO: check where exactly to start pixel fetching for objects
        while t_counter < passed_t_cycles {
            let (next_step, spent_cycles) = self.step_subtask(bus);

            self.current_step = next_step;

            for _ in (0..spent_cycles) {
                let pixel_pushed = self.try_push_pixel_to_screen(bus, current_frame);

                if matches!(self.current_step, PixelFetcherStep::GetTile) {
                    if let Some(object) = self.get_object_for_current_position(object_buffer) {
                        self.set_object_fetch_mode(object);
                    } else {
                        self.fetch_mode = match self.fetch_mode {
                            FetchMode::Object(_) => FetchMode::Background,
                            _ => self.fetch_mode,
                        };
                    }
                }

                if pixel_pushed && self.window_reached(bus) {
                    self.set_window_fetch_mode();
                    break;
                }
            }

            t_counter += spent_cycles;
        }
    }

    fn step_subtask(&mut self, bus: &Bus) -> (PixelFetcherStep, u8) {
        match self.current_step {
            PixelFetcherStep::Sleep(counter) => {
                if counter == 1 {
                    (PixelFetcherStep::GetTile, 1)
                } else {
                    (PixelFetcherStep::Sleep(counter - 1), 1)
                }
            }
            PixelFetcherStep::GetTile => {
                self.tile_number = self.fetch_tile_number(bus);
                (PixelFetcherStep::GetTileDataLow, 2)
            }
            PixelFetcherStep::GetTileDataLow => {
                self.tile_data_low = self.fetch_tile_data_low(bus);
                (PixelFetcherStep::GetTileDataHigh, 2)
            }
            PixelFetcherStep::GetTileDataHigh => {
                self.tile_data_high = self.fetch_tile_data_high(bus);
                (PixelFetcherStep::Push, 2)
            }
            PixelFetcherStep::Push => {
                let push_is_allowed = match self.fetch_mode {
                    FetchMode::Object(_) => true,
                    FetchMode::Background | FetchMode::Window => self.background_queue.is_empty(),
                };

                if push_is_allowed {
                    self.push_pixel_data_to_queue();
                    (PixelFetcherStep::GetTile, 2)
                } else {
                    (PixelFetcherStep::Push, 1)
                }
            }
        }
    }

    // TODO: merge with other step_subtask method if possible
    fn step_subtask_object(&mut self, bus: &Bus) -> (PixelFetcherStep, u8) {
        match self.current_step {
            PixelFetcherStep::Sleep(counter) => {
                if counter == 1 {
                    (PixelFetcherStep::GetTile, 1)
                } else {
                    (PixelFetcherStep::Sleep(counter - 1), 1)
                }
            }
            PixelFetcherStep::GetTile => {
                self.tile_number = self.fetch_tile_number(bus);
                (PixelFetcherStep::GetTileDataLow, 2)
            }
            PixelFetcherStep::GetTileDataLow => {
                self.tile_data_low = self.fetch_tile_data_low(bus);
                (PixelFetcherStep::GetTileDataHigh, 2)
            }
            PixelFetcherStep::GetTileDataHigh => {
                self.tile_data_high = self.fetch_tile_data_high(bus);
                (PixelFetcherStep::Push, 2)
            }
            PixelFetcherStep::Push => {
                let push_is_allowed = match self.fetch_mode {
                    FetchMode::Object(_) => true,
                    FetchMode::Background | FetchMode::Window => self.background_queue.is_empty(),
                };

                if push_is_allowed {
                    self.push_pixel_data_to_queue();
                    (PixelFetcherStep::GetTile, 2)
                } else {
                    (PixelFetcherStep::Push, 1)
                }
            }
        }
    }

    /// Get the tile number used as an index for accessing the tile data from vram in the next two
    /// steps:
    ///   * Background & Window: Access the current 32x32 tile map and return the tile index based
    ///     on the current position. The current tile map is detected internally using the LCDC
    ///     register.
    ///   * Object: Read tile number from current object
    fn fetch_tile_number(&self, bus: &Bus) -> u8 {
        match self.fetch_mode {
            FetchMode::Background => {
                let x_offset: u16 =
                    ((self.fetcher_x.wrapping_add(bus.get_scroll_x() / TILE_SIZE)) % 32).into();
                let y_offset: u16 =
                    ((self.current_line.wrapping_add(bus.get_scroll_y())) / TILE_SIZE).into();

                let tilemap_address = bus.get_bg_tile_map().start() + (y_offset * 32) + x_offset;

                bus.ppu_read(tilemap_address)
            }
            FetchMode::Window => {
                let x_offset: u16 = self.fetcher_x.into();
                let y_offset: u16 = (self.window_line_counter / TILE_SIZE).into();

                let tilemap_address =
                    bus.get_window_tile_map().start() + (y_offset * 32) + x_offset;

                bus.ppu_read(tilemap_address)
            }
            FetchMode::Object(object_attribute) => object_attribute.tile_index,
        }
    }

    /// Returns the lower byte of the tile at the address pointed to by the tile map at the given
    /// index.
    fn fetch_tile_data_low(&mut self, bus: &Bus) -> u8 {
        let address = match self.fetch_mode {
            FetchMode::Background | FetchMode::Window => bus
                .get_bg_window_tile_data_area()
                .get_tile_address(self.tile_number),
            FetchMode::Object(_) => bus
                .get_object_tile_data_area()
                .get_tile_address(self.tile_number),
        };

        let offset = 2
            * (match self.fetch_mode {
                FetchMode::Background => self.current_line.wrapping_add(bus.get_scroll_y()),
                FetchMode::Window => self.window_line_counter,
                FetchMode::Object(_) => 0,
            } % TILE_SIZE);

        self.tile_data_address = address + offset as u16;

        bus.ppu_read(self.tile_data_address)
    }

    /// Read the next byte at the address set in `fetch_tile_data_low`.
    /// Returns the higher byte of the tile at the address pointed to by the tile map at the given
    /// index.
    fn fetch_tile_data_high(&self, bus: &Bus) -> u8 {
        bus.ppu_read(self.tile_data_address + 1)
    }

    /// Iterate through the current row of the fetched tile and push combined pixel data to the
    /// pixel fifo queue (`self.background_queue`).
    fn push_pixel_data_to_queue(&mut self) {
        for i in 0..TILE_SIZE {
            let offset = 7 - i;
            let high = (self.tile_data_high >> offset) & 1;
            let low = (self.tile_data_low >> offset) & 1;

            let pixel = (high << 1) | low;

            match self.fetch_mode {
                FetchMode::Background | FetchMode::Window => self.background_queue.push(pixel),
                FetchMode::Object(_) => self.sprite_queue.push(pixel),
            };
        }

        self.fetcher_x += 1;
    }

    /// Tries to push pixels from the queues to the current frame.
    fn try_push_pixel_to_screen(&mut self, bus: &Bus, frame: &mut PixelData) -> bool {
        if self.discard_counter > 0 && self.background_queue.pop().is_ok() {
            self.discard_counter -= 1;
            return false;
        }

        let Ok(bg_pixel) = self.background_queue.pop() else {
            return false;
        };

        if self.render_x > 159 {
            return false;
        }

        let bg_pixel = if bus.bg_window_enabled() { bg_pixel } else { 0 };

        let final_pixel = match self.sprite_queue.pop() {
            Ok(sprite_pixel) => {
                if sprite_pixel == 0 {
                    bg_pixel
                } else {
                    sprite_pixel
                }
            }
            Err(_) => bg_pixel,
        };

        let index = ((self.current_line as usize) * LCD_WIDTH + (self.render_x as usize));
        frame.0[index] = final_pixel;

        self.render_x += 1;

        true
    }

    fn window_reached(&self, bus: &Bus) -> bool {
        if !bus.window_enabled() {
            return false;
        }

        if bus.get_window_y() > self.current_line {
            return false;
        }

        if bus.get_window_x() > self.render_x {
            return false;
        }

        true
    }

    fn set_window_fetch_mode(&mut self) {
        if matches!(self.fetch_mode, FetchMode::Window) {
            return;
        }

        self.fetch_mode = FetchMode::Window;
        self.fetcher_x = 0;
        self.current_step = PixelFetcherStep::GetTile;
        self.background_queue.clear();
    }

    fn get_object_for_current_position(
        &self,
        object_buffer: &ObjectBuffer,
    ) -> Option<ObjectAttribute> {
        object_buffer
            .iter()
            .find(|&object| {
                object.x_position <= self.render_x + 8 && object.x_position > self.render_x
            })
            .map(|object| *object)

        // None
    }

    fn set_object_fetch_mode(&mut self, object_attribute: ObjectAttribute) {
        if matches!(self.fetch_mode, FetchMode::Object(_)) {
            return;
        }

        self.fetch_mode = FetchMode::Object(object_attribute);
        self.current_step = PixelFetcherStep::GetTile;
    }
}
