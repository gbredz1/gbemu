pub(crate) mod timer_bus;

use crate::bus::Interrupt;
use crate::timer::timer_bus::TAC;
use timer_bus::TimerBus;

pub(crate) const DMG_DIV_INITIAL_VALUE: u8 = 0xD3;

#[derive(Default)]
pub struct Timer {
    div_cycles: u16,
    timer_cycles: u16,
}

impl Timer {
    pub fn reset(&mut self, bus: &mut impl TimerBus) {
        bus.set_div(DMG_DIV_INITIAL_VALUE);
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

        let timer_freq = match (tac.contains(TAC::ClockSelect1), tac.contains(TAC::ClockSelect0)) {
            (false, false) => 256, // 4096 Hz   (00)
            (false, true) => 4,    // 262144 Hz (01)
            (true, false) => 16,   // 65536 Hz  (10)
            (true, true) => 64,    // 16384 Hz  (11)
        };

        // Update TIMA according to selected frequency
        self.timer_cycles = self.timer_cycles.wrapping_add(cycles as u16);

        if self.timer_cycles >= timer_freq {
            self.timer_cycles -= timer_freq;

            let tima = bus.tima();
            if tima == 0xFF {
                // Overflow
                bus.set_tima(bus.tma()); // put TMA into TIMA
                bus.set_interrupt_flag(Interrupt::TIMER); // Trigger TIMER interrupt
            } else {
                // Increment TIMA
                bus.set_tima(tima.wrapping_add(1));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::InterruptBus;

    use crate::tests::bus::TestBus;

    impl TimerBus for TestBus {}

    #[test]
    fn test_div_increment() {
        let mut timer = Timer::default();
        let mut bus = TestBus::default();

        timer.step(&mut bus, 255);
        assert_eq!(bus.div(), 0);

        timer.step(&mut bus, 1);
        assert_eq!(bus.div(), 1);
    }

    #[test]
    fn test_tima_frequencies() {
        let run_test = |tac: TAC, cycles: usize| {
            let mut timer = Timer::default();
            let mut bus = TestBus::default();
            bus.set_tac(tac);
            timer.step(&mut bus, (cycles - 1) as u8);
            assert_eq!(bus.tima(), 0);
            timer.step(&mut bus, 1);
            assert_eq!(bus.tima(), 1);
        };

        run_test(TAC::Enable, 256); // Test 4096 Hz (256 cycles)
        run_test(TAC::Enable | TAC::ClockSelect0, 4); // Test 262144 Hz (4 cycles)
        run_test(TAC::Enable | TAC::ClockSelect1, 16); // Test 65536 Hz (16 cycles)
        run_test(TAC::Enable | TAC::ClockSelect1 | TAC::ClockSelect0, 64); // Test 16384 Hz (64 cycles)
    }

    #[test]
    fn test_tima_overflow() {
        let mut timer = Timer::default();
        let mut bus = TestBus::default();

        bus.set_tac(TAC::Enable); // Enable timer, freq 00
        bus.set_tima(0xFF);
        bus.set_tma(0x42);

        timer.step(&mut bus, 255);
        assert_eq!(bus.tima(), 0xFF);
        timer.step(&mut bus, 1);
        assert_eq!(bus.tima(), 0x42);
        assert!(bus.interrupt_flag().contains(Interrupt::TIMER));
    }

    #[test]
    fn test_timer_disabled() {
        let mut timer = Timer::default();
        let mut bus = TestBus::default();

        // Make sure the timer is disabled.
        bus.set_tac(TAC::empty());
        bus.set_tima(0x42);

        // Run enough cycles that TIMA should have incremented if enabled
        timer.step(&mut bus, 255);
        timer.step(&mut bus, 1);

        // Check that TIMA has not changed.
        assert_eq!(bus.tima(), 0x42);

        // Check that DIV continues to increment even if the timer is disabled.
        assert_eq!(bus.div(), 1);
    }
}
