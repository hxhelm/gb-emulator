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

#[derive(Clone, Copy, Default)]
pub enum PPUMode {
    #[default]
    OBJSearch,
    SendPixels,
    HorizontalBlank,
    VerticalBlank,
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
            mode: PPUMode::VerticalBlank,
            mode_timer: CYCLES_PER_LINE,
            current_frame: PixelData::default(),
            pixel_fetcher: PixelFetcher::init(),
            screen_finished: false,
            scanline_x_scroll: 0,
        }
    }

    fn check_mode_change(self) -> (bool, u16) {
        let send_pixel_timer: u16 = (172 + self.scanline_x_scroll % 8).into();
        let hblank_timer: u16 = (204 - self.scanline_x_scroll % 8).into();
        let timer = self.mode_timer;

        match self.mode {
            PPUMode::OBJSearch if timer >= 80 => (true, timer - 80),
            // TODO: account for this modes penalties, see: https://gbdev.io/pandocs/Rendering.html#mode-3-length
            PPUMode::SendPixels if timer >= send_pixel_timer => (true, timer - send_pixel_timer),
            PPUMode::HorizontalBlank if timer >= hblank_timer => (true, timer - hblank_timer),
            PPUMode::VerticalBlank if timer >= CYCLES_PER_LINE => (true, timer - CYCLES_PER_LINE),
            _ => (false, 0),
        }
    }

    fn change_mode(&mut self, bus: &mut Bus) {
        self.mode = match self.mode {
            PPUMode::OBJSearch => {
                self.scanline_x_scroll = bus.get_scroll_x();
                PPUMode::SendPixels
            }
            PPUMode::SendPixels => PPUMode::HorizontalBlank,
            PPUMode::HorizontalBlank => {
                bus.update_line();

                if bus.current_line() == 144 {
                    bus.request_vblank_interrupt();
                    PPUMode::VerticalBlank
                } else {
                    self.pixel_fetcher.reset_line(bus);
                    PPUMode::OBJSearch
                }
            }
            PPUMode::VerticalBlank => {
                bus.update_line();

                if bus.current_line() == 0 {
                    self.screen_finished = true;
                    self.pixel_fetcher.reset_frame(bus);
                    PPUMode::OBJSearch
                } else {
                    self.pixel_fetcher.reset_line(bus);
                    PPUMode::VerticalBlank
                }
            }
        };
        self.update_stat_mode(bus);
    }

    fn update_stat_mode(&self, bus: &mut Bus) {
        bus.lcd_status_set_mode(self.mode);

        let stat_conditions = bus.lcd_status_condition();
        let trigger_interrupt = match self.mode {
            PPUMode::OBJSearch if stat_conditions.mode2 => true,
            PPUMode::VerticalBlank if stat_conditions.mode1 => true,
            PPUMode::HorizontalBlank if stat_conditions.mode0 => true,
            _ => false,
        };

        if trigger_interrupt {
            bus.request_stat_interrupt();
        }
    }

    pub(crate) fn step(&mut self, t_cycles: u8, bus: &mut Bus) -> Option<PixelData> {
        if !bus.lcd_enabled() {
            return None;
        }

        self.mode_timer = self.mode_timer.saturating_add(t_cycles.into());

        let (should_change_mode, remaining_cycles) = self.check_mode_change();

        if matches!(self.mode, PPUMode::SendPixels) && remaining_cycles != 0 {
            self.pixel_fetcher
                .step(bus, remaining_cycles as u8, &mut self.current_frame);
        }

        if should_change_mode {
            self.mode_timer = remaining_cycles;
            self.change_mode(bus);
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
            framebuffer
        } else {
            None
        }
    }
}
