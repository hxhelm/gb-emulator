#![allow(unused)]
use std::collections::VecDeque;

use crate::memory::bus::Bus;

use super::{PixelData, LCD_WIDTH};

const SCROLL_X: u16 = 0xFF43;
const SCROLL_Y: u16 = 0xFF42;
const WINDOW_X: u16 = 0xFF4B;
const WINDOW_Y: u16 = 0xFF4A;

impl Bus {
    /// Calculate the bottom right coordinate of the viewport using the scroll registers.
    fn get_bottom_right_vp(&self) -> (u16, u16) {
        let x = self.get_scroll_x() as u16;
        let y = self.get_scroll_y() as u16;
        ((y + 143) % 256, (x + 159) % 256)
    }
}

#[derive(Copy, Clone)]
struct Fifo {
    queue: [u8; 16],
    index: usize,
    length: usize,
}

impl Fifo {
    pub fn new() -> Self {
        Self {
            queue: [0; 16],
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
        self.discard_counter = bus.get_scroll_x() % 8;
        eprintln!("FIFO line reset!, Line: {}", self.current_line);
    }

    pub fn step(&mut self, bus: &mut Bus, passed_t_cycles: u8, current_frame: &mut PixelData) {
        let mut t_counter = 0;

        while t_counter < passed_t_cycles {
            let (next_step, spent_cycles) = match self.current_step {
                PixelFetcherStep::Sleep(counter) => {
                    // eprintln!("PixelFetcher::Sleep");
                    if counter == 1 {
                        (PixelFetcherStep::GetTile, 1)
                    } else {
                        (PixelFetcherStep::Sleep(counter - 1), 1)
                    }
                }
                PixelFetcherStep::GetTile => {
                    // eprintln!("PixelFetcher::GetTile");
                    self.tile_number = self.fetch_tile_number(bus);
                    (PixelFetcherStep::GetTileDataLow, 2)
                }
                PixelFetcherStep::GetTileDataLow => {
                    // eprintln!("PixelFetcher::DataLow");
                    self.tile_data_low = self.fetch_tile_data_low(bus);
                    (PixelFetcherStep::GetTileDataHigh, 2)
                }
                PixelFetcherStep::GetTileDataHigh => {
                    // eprintln!("PixelFetcher::DataHigh");
                    self.tile_data_high = self.fetch_tile_data_high(bus);
                    (PixelFetcherStep::Push, 2)
                }
                PixelFetcherStep::Push => {
                    if self.background_queue.is_empty() {
                        // eprintln!("PixelFetcher::Push::PushedToQueue");
                        self.background_queue.push(self.get_pixel_from_tile());
                        self.fetcher_x = (self.fetcher_x + 1) % 32;

                        (PixelFetcherStep::GetTile, 2)
                    } else {
                        // eprintln!("PixelFetcher::Push::QueueNotEmpty");
                        (PixelFetcherStep::Push, 1)
                    }
                }
            };

            self.current_step = next_step;

            for _ in (0..=spent_cycles) {
                self.try_push_pixel_to_screen(current_frame);
            }

            t_counter += spent_cycles;
        }
    }

    fn fetch_tile_number(&self, bus: &Bus) -> u8 {
        // TODO: window handling

        let x_offset: u16 = ((self.fetcher_x.wrapping_add(bus.get_scroll_x() / 8)) % 32).into();
        let y_offset: u16 = ((self.current_line.wrapping_add(bus.get_scroll_y())) / 8).into();

        let tilemap_start_address = bus.get_bg_tile_map().start();

        let tilemap_address = tilemap_start_address + (y_offset * 32) + x_offset;

        let tile_number = bus.read_byte_unchecked(tilemap_address);

        eprintln!(
            "TMS: 0x{:04X}, TMA: 0x{:04X}, TN: {}",
            tilemap_start_address, tilemap_address, tile_number
        );

        tile_number
    }

    /// Access the current 32x32 tile map by their index. The current tile map is detected
    /// internally using the LCDC register.
    /// Returns the lower byte of the tile at the address pointed to by the tile map at the given
    /// index.
    fn fetch_tile_data_low(&mut self, bus: &Bus) -> u8 {
        let address_raw = bus
            .get_bg_window_tile_data_area()
            .get_tile_address(self.tile_number);

        self.tile_data_address = address_raw as u16;

        let data = bus.read_byte_unchecked(self.tile_data_address);

        eprintln!("TAddr: {:04X}, TData: {:04X}", self.tile_data_address, data);

        data
    }

    /// Read the next byte at the address set in `fetch_tile_data_low`
    /// Returns the higher byte of the tile at the address pointed to by the tile map at the given
    /// index.
    fn fetch_tile_data_high(&self, bus: &Bus) -> u8 {
        let addr_high = self.tile_data_address + 1;
        let data = bus.read_byte_unchecked(addr_high);

        eprintln!("TAddr: {:04X}, TData: {:04X}", addr_high, data);

        data
    }

    fn get_pixel_from_tile(&self) -> u8 {
        let offset = self.fetcher_x % 8;
        let bit_mask = 0b00000001;

        let low = (self.tile_data_low & bit_mask) >> offset;
        let high = (self.tile_data_high & bit_mask) >> offset;

        low | (high << 1)
    }

    fn try_push_pixel_to_screen(&mut self, frame: &mut PixelData) {
        if self.discard_counter > 0 {
            if self.background_queue.pop().is_ok() {
                // eprintln!("PixelFetcher::Pixel::Discard");

                self.discard_counter -= 1;
                self.render_x += 1;
            }

            return;
        }

        let Ok(bg_pixel) = self.background_queue.pop() else {
            return;
        };

        // eprintln!("PixelFetcher::Pixel::PushedToScreen");
        let index = ((self.current_line as usize) * LCD_WIDTH + (self.render_x as usize));
        frame.0[index] = bg_pixel;

        if bg_pixel != 0 {
            eprintln!("Wrote non-zero pixel! {:04X}", bg_pixel);
        }

        self.render_x += 1;
    }
}
