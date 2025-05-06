pub(crate) mod timer_bus;

use crate::bus::Interrupt;
use crate::timer::timer_bus::TAC;
use timer_bus::TimerBus;

#[derive(Default)]
pub struct Timer {
    div_cycles: u16,
    timer_cycles: u16,
}

impl Timer {
    pub fn reset(&mut self, bus: &mut impl TimerBus) {
        bus.set_div(0x00);
        bus.set_tima(0x00);
        bus.set_tma(0x00);
        bus.set_tac_u8(0xF8);
        self.div_cycles = 0;
        self.timer_cycles = 0;
    }

    pub fn step(&mut self, bus: &mut impl TimerBus, cycles: u8) {
        // Update DIV register (increments every 256 CPU cycles)
        self.div_cycles = self.div_cycles.wrapping_add(cycles as u16);
        if self.div_cycles >= 256 {
            self.div_cycles -= 256;
            bus.set_div(bus.div().wrapping_add(1));
        }

        // Check if timer is enabled (TAC)
        let tac = bus.tac();
        if !tac.contains(TAC::Enable) {
            return;
        }

        let timer_freq = match (tac.contains(TAC::ClockSelect0), tac.contains(TAC::ClockSelect1)) {
            (false, true) => 16,    // 262144 Hz
            (true, false) => 64,    // 65536 Hz
            (true, true) => 256,    // 16384 Hz
            (false, false) => 1024, // 4096 Hz
        };

        // Update TIMA according to selected frequency
        self.timer_cycles = self.timer_cycles.wrapping_add(cycles as u16);

        if self.timer_cycles >= timer_freq {
            self.timer_cycles -= timer_freq;

            // Increment TIMA and check for overflow
            let (tima, overflow) = bus.tima().overflowing_add(1);

            // If TIMA overflows
            if overflow {
                bus.set_tima(bus.tma());

                // Trigger TIMER interrupt
                bus.set_interrupt_flag(Interrupt::TIMER);
            } else {
                bus.set_tima(tima);
            }
        }
    }
}
