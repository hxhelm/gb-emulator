use crate::cpu::CPU;

#[derive(Default, Clone, Copy)]
pub struct Clock {
    pub(crate) t: u64,
    pub(crate) m: u64,
}

impl Clock {
    pub(crate) fn increment(&mut self, t_cycles: u8) {
        assert_eq!(t_cycles % 4, 0, "t_cycles need to be a multiple of 4");
        self.t = self.m.wrapping_add(t_cycles.into());
        self.m = self.t / 4;
    }
}

impl CPU {
    fn read_tac(&self) -> u8 {
        self.bus.read_byte(0xFF07)
    }

    fn is_timer_enabled(&self) -> bool {
        (self.read_tac() & 0x04) != 0
    }

    fn get_timer_t_frequency(&self) -> u64 {
        const TIMER_FREQUENCIES: [u64; 4] = [256, 4, 16, 64];
        TIMER_FREQUENCIES[(self.read_tac() & 0x03) as usize]
    }

    fn read_tma(&self) -> u8 {
        self.bus.read_byte(0xFF06)
    }

    fn read_tima(&self) -> u8 {
        self.bus.read_byte(0xFF05)
    }

    fn write_tima(&mut self, value: u8) {
        self.bus.write_byte(0xFF05, value);
    }

    pub(crate) fn update_timers(&mut self) {
        if !self.is_timer_enabled() {
            return;
        }

        let timer_frequency = self.get_timer_t_frequency();

        let timer_diff = self.clock.t.saturating_sub(self.last_timer_update);
        if timer_diff >= timer_frequency {
            let increments = timer_diff / timer_frequency;
            let (tima, did_overflow) = self.read_tima().overflowing_add(increments as u8);

            if did_overflow {
                self.write_tima(self.read_tma());
                // self.bus.request_timer_interrupt();
            } else {
                self.write_tima(tima);
            }

            self.last_timer_update += increments * timer_frequency;
        }
    }
}
