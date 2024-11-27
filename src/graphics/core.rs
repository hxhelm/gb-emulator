use crate::memory::bus::Bus;

const CYCLES_PER_LINE: u16 = 456;

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

impl PPUMode {
    const fn should_change_mode(self, timer: u16) -> bool {
        match self {
            Self::OBJSearch if timer >= 80 => true,
            // TODO: account for this modes penalties, see: https://gbdev.io/pandocs/Rendering.html#mode-3-length
            Self::SendPixels if timer >= 172 => true,
            Self::HorizontalBlank if timer >= 204 => true,
            Self::VerticalBlank if timer >= CYCLES_PER_LINE => true,
            _ => false,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct PPU {
    mode: PPUMode,
    mode_timer: u16,
}

impl PPU {
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
                if current_line == 153 {
                    PPUMode::VerticalBlank
                } else {
                    PPUMode::OBJSearch
                }
            }
        };
    }

    pub(crate) fn step(&mut self, t_cycles: u8, bus: &mut Bus) {
        self.mode_timer = self.mode_timer.saturating_add(t_cycles.into());

        if self.mode.should_change_mode(self.mode_timer) {
            self.mode_timer = 0;
            self.change_mode(bus.lcd_current_line());

            if matches!(self.mode, PPUMode::OBJSearch | PPUMode::VerticalBlank) {
                bus.lcd_update_line();
            }

            bus.update_ppu_mode(self.mode);
        }

        // TODO: act according to mode, maybe implement inside change_mode?
    }
}
