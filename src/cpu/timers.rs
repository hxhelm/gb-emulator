use crate::cpu::CPU;
use crate::memory::bus::{TIMER_CONTROL, TIMER_COUNTER, TIMER_MODULO};

#[derive(Default, Clone, Copy)]
pub struct Clock {
    pub(crate) t: u64,
    pub(crate) m: u64,
}

impl Clock {
    pub(crate) fn increment(&mut self, t_cycles: u8) {
        assert_eq!(t_cycles % 4, 0, "t_cycles need to be a multiple of 4");
        self.t = self.t.wrapping_add(t_cycles.into());
        self.m = self.t / 4;
    }
}

impl CPU {
    fn read_tac(&self) -> u8 {
        self.bus.read_byte(TIMER_CONTROL)
    }

    fn is_timer_enabled(&self) -> bool {
        (self.read_tac() & 0x04) != 0
    }

    fn get_timer_m_frequency(&self) -> u64 {
        const TIMER_FREQUENCIES: [u64; 4] = [256, 4, 16, 64];
        TIMER_FREQUENCIES[(self.read_tac() & 0x03) as usize]
    }

    fn read_tma(&self) -> u8 {
        self.bus.read_byte(TIMER_MODULO)
    }

    fn read_tima(&self) -> u8 {
        self.bus.read_byte(TIMER_COUNTER)
    }

    fn write_tima(&mut self, value: u8) {
        self.bus.write_byte(TIMER_COUNTER, value);
    }

    pub(crate) fn update_timers(&mut self, t_cycles: u8) {
        self.clock.increment(t_cycles);

        if !self.is_timer_enabled() {
            return;
        }

        let timer_frequency = self.get_timer_m_frequency();
        let timer_diff = self.clock.m.saturating_sub(self.last_timer_update);

        if timer_diff >= timer_frequency {
            let increments = timer_diff / timer_frequency;
            let (tima, did_overflow) = self.read_tima().overflowing_add(increments as u8);

            if did_overflow {
                self.write_tima(self.read_tma());
                self.bus.request_timer_interrupt();
            } else {
                self.write_tima(tima);
            }

            self.last_timer_update += increments * timer_frequency;
        }
    }
}
