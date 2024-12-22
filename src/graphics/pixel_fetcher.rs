#![allow(unused)]
use std::collections::VecDeque;

use crate::memory::bus::Bus;

use super::{fifo::Fifo, PixelData, LCD_WIDTH};

const SCROLL_X: u16 = 0xFF43;
const SCROLL_Y: u16 = 0xFF42;
const WINDOW_X: u16 = 0xFF4B;
const WINDOW_Y: u16 = 0xFF4A;
const TILE_SIZE: u8 = 8;

impl Bus {
    /// Calculate the bottom right coordinate of the viewport using the scroll registers.
    fn get_bottom_right_vp(&self) -> (u16, u16) {
        let x = self.get_scroll_x() as u16;
        let y = self.get_scroll_y() as u16;
        ((y + 143) % 256, (x + 159) % 256)
    }
}

#[derive(Clone, Copy)]
enum PixelFetcherStep {
    GetTile,
    GetTileDataLow,
    GetTileDataHigh,
    Sleep(u8),
    Push,
}

#[derive(Clone, Copy)]
pub(crate) struct PixelFetcher {
    current_step: PixelFetcherStep,
    current_line: u8,
    /// Tile number fetching
    fetcher_x: u8,
    is_fetching_window: bool,
    window_line_counter: u8,
    tile_number: u8,
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
            fetcher_x: 0,
            is_fetching_window: false,
            window_line_counter: 0,
            tile_number: 0,
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
        *self = Self::init();
        self.current_line = bus.lcd_current_line();
        self.discard_counter = bus.get_scroll_x() % TILE_SIZE;
    }

    pub fn step(&mut self, bus: &mut Bus, passed_t_cycles: u8, current_frame: &mut PixelData) {
        let mut t_counter = 0;

        while t_counter < passed_t_cycles {
            let (next_step, spent_cycles) = match self.current_step {
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
                    if self.background_queue.is_empty() {
                        self.push_pixel_data_to_queue();
                        (PixelFetcherStep::GetTile, 2)
                    } else {
                        (PixelFetcherStep::Push, 1)
                    }
                }
            };

            self.current_step = next_step;

            for _ in (0..spent_cycles) {
                self.try_push_pixel_to_screen(current_frame);
            }

            t_counter += spent_cycles;
        }
    }

    fn fetch_tile_number(&self, bus: &Bus) -> u8 {
        // TODO: window handling

        let x_offset: u16 =
            ((self.fetcher_x.wrapping_add(bus.get_scroll_x() / TILE_SIZE)) % 32).into();
        let y_offset: u16 =
            ((self.current_line.wrapping_add(bus.get_scroll_y())) / TILE_SIZE).into();

        let tilemap_address = bus.get_bg_tile_map().start() + (y_offset * 32) + x_offset;
        bus.read_byte_unchecked(tilemap_address)
    }

    /// Access the current 32x32 tile map by their index. The current tile map is detected
    /// internally using the LCDC register.
    /// Returns the lower byte of the tile at the address pointed to by the tile map at the given
    /// index.
    fn fetch_tile_data_low(&mut self, bus: &Bus) -> u8 {
        let address = bus
            .get_bg_window_tile_data_area()
            .get_tile_address(self.tile_number);

        let offset = 2 * (self.current_line.wrapping_add(bus.get_scroll_y()) % TILE_SIZE);
        self.tile_data_address = address + offset as u16;

        bus.read_byte_unchecked(self.tile_data_address)
    }

    /// Read the next byte at the address set in `fetch_tile_data_low`
    /// Returns the higher byte of the tile at the address pointed to by the tile map at the given
    /// index.
    fn fetch_tile_data_high(&self, bus: &Bus) -> u8 {
        bus.read_byte_unchecked(self.tile_data_address + 1)
    }

    fn push_pixel_data_to_queue(&mut self) {
        for i in 0..TILE_SIZE {
            let offset = 7 - i;
            let high = (self.tile_data_high >> offset) & 1;
            let low = (self.tile_data_low >> offset) & 1;
            self.background_queue.push((high << 1) | low);
        }

        self.fetcher_x += 1;
    }

    fn try_push_pixel_to_screen(&mut self, frame: &mut PixelData) {
        if self.discard_counter > 0 && self.background_queue.pop().is_ok() {
            self.discard_counter -= 1;
            return;
        }

        let Ok(bg_pixel) = self.background_queue.pop() else {
            return;
        };

        if self.render_x > 159 {
            return;
        }

        let index = ((self.current_line as usize) * LCD_WIDTH + (self.render_x as usize));
        frame.0[index] = bg_pixel;

        self.render_x += 1;
    }
}
