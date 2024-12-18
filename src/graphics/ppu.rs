use crate::memory::bus::Bus;

use super::pixel_fetcher::PixelFetcher;

/// TODO: get winit/pixels working and start paving the way for rendering:
///  1. winit/pixels window running ✅
///  2. read tiles/sprites from memory
///  3. ???
///  4. draw things to screen

pub const LCD_WIDTH: usize = 160;
pub const LCD_HEIGHT: usize = 144;
const CYCLES_PER_LINE: u16 = 456;

#[derive(Clone, Copy)]
pub struct PixelData(pub [u8; LCD_WIDTH * LCD_HEIGHT]);

impl Default for PixelData {
    fn default() -> Self {
        Self([0; LCD_WIDTH * LCD_HEIGHT])
    }
}

#[derive(Clone, Copy)]
pub enum PPUMode {
    OBJSearch,
    SendPixels,
    HorizontalBlank,
    VerticalBlank,
}

impl Default for PPUMode {
    fn default() -> Self {
        Self::OBJSearch
    }
}

#[derive(Clone, Copy)]
pub struct PPU {
    mode: PPUMode,
    mode_timer: u16,
    current_frame: PixelData,
    pixel_fetcher: PixelFetcher,
    screen_finished: bool,
    scanline_x_scroll: u8,
}

impl PPU {
    pub fn init() -> PPU {
        Self {
            mode: PPUMode::OBJSearch,
            mode_timer: 0,
            current_frame: PixelData::default(),
            pixel_fetcher: PixelFetcher::init(),
            screen_finished: false,
            scanline_x_scroll: 0,
        }
    }

    fn check_mode_change(self, timer: u16) -> (bool, u16) {
        let send_pixel_timer: u16 = (172 + self.scanline_x_scroll % 8).into();

        match self.mode {
            PPUMode::OBJSearch if timer >= 80 => (true, timer - 80),
            // TODO: account for this modes penalties, see: https://gbdev.io/pandocs/Rendering.html#mode-3-length
            PPUMode::SendPixels if timer >= send_pixel_timer => (true, timer - send_pixel_timer),
            PPUMode::HorizontalBlank if timer >= 204 => (true, timer - 204),
            PPUMode::VerticalBlank if timer >= CYCLES_PER_LINE => (true, timer - CYCLES_PER_LINE),
            _ => (false, 0),
        }
    }

    fn change_mode(&mut self, current_line: u8) {
        self.mode = match self.mode {
            PPUMode::OBJSearch => PPUMode::SendPixels,
            PPUMode::SendPixels => PPUMode::HorizontalBlank,
            PPUMode::HorizontalBlank => {
                if current_line == 143 {
                    PPUMode::VerticalBlank
                } else {
                    PPUMode::OBJSearch
                }
            }
            PPUMode::VerticalBlank => {
                if current_line <= 153 {
                    PPUMode::VerticalBlank
                } else {
                    self.screen_finished = true;
                    PPUMode::OBJSearch
                }
            }
        };
    }

    pub(crate) fn step(&mut self, t_cycles: u8, bus: &mut Bus) -> Option<PixelData> {
        self.mode_timer = self.mode_timer.saturating_add(t_cycles.into());

        let (should_change_mode, remaining_cycles) = self.check_mode_change(self.mode_timer);
        if should_change_mode {
            self.mode_timer = remaining_cycles;

            if matches!(self.mode, PPUMode::SendPixels) {
                self.pixel_fetcher
                    .step(bus, remaining_cycles as u8, &mut self.current_frame);
            }

            self.change_mode(bus.lcd_current_line());

            if matches!(self.mode, PPUMode::OBJSearch | PPUMode::VerticalBlank) {
                self.update_scanline(bus);
            }

            bus.update_ppu_mode(self.mode);
        }

        if matches!(self.mode, PPUMode::SendPixels) {
            self.pixel_fetcher
                .step(bus, t_cycles, &mut self.current_frame);
        }

        if self.screen_finished {
            let framebuffer = Some(self.current_frame);
            self.current_frame = PixelData::default();
            self.screen_finished = false;
            eprintln!("===Screen finished===");
            framebuffer
        } else {
            None
        }
    }

    fn update_scanline(&mut self, bus: &mut Bus) {
        bus.lcd_update_line();
        self.scanline_x_scroll = bus.get_scroll_x();
        self.pixel_fetcher.reset_line(bus);
    }
}
