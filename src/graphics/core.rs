use crate::memory::bus::Bus;

const CYCLES_PER_LINE: u16 = 456;
const CYCLES_PER_VERTICAL_BLANK: u16 = 4560;

#[derive(Clone, Copy)]
pub(crate) enum PPUMode {
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
    fn should_change_mode(&self, timer: u16) -> bool {
        match self {
            Self::OBJSearch if timer >= 80 => true,
            // TODO: account for this modes penalties, see: https://gbdev.io/pandocs/Rendering.html#mode-3-length
            Self::SendPixels if timer >= 172 => true,
            Self::HorizontalBlank if timer >= 204 => true,
            Self::VerticalBlank if timer >= CYCLES_PER_VERTICAL_BLANK => true,
            _ => false,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub(crate) struct PPU {
    mode: PPUMode,
    mode_timer: u16,
    current_line: u8,
}

impl PPU {
    fn change_mode(&mut self) {
        self.mode = match self.mode {
            PPUMode::OBJSearch => PPUMode::SendPixels,
            PPUMode::SendPixels => {
                // TODO: send pixel data
                PPUMode::HorizontalBlank
            }
            PPUMode::HorizontalBlank => {
                if self.current_line == 143 {
                    PPUMode::VerticalBlank
                } else {
                    PPUMode::OBJSearch
                }
            }
            PPUMode::VerticalBlank => {
                self.current_line = 0;
                PPUMode::OBJSearch
            }
        };
    }

    pub(crate) fn step(&mut self, t_cycles: u8, bus: &mut Bus) {
        self.mode_timer = self.mode_timer.saturating_add(t_cycles.into());

        if self.mode_timer >= CYCLES_PER_LINE {
            self.current_line += 1;
        }

        if self.mode.should_change_mode(self.mode_timer) {
            self.mode_timer = 0;
            self.change_mode();
        }

        bus.update_ppu_mode(&self.mode);

        // TODO: act according to mode, maybe implement inside change_mode?
    }
}
