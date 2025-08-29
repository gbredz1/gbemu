#[cfg(any(test, feature = "test-bus"))]
pub(crate) mod bus {
    use crate::bus::{BusIO, InterruptBus};
    use crate::CpuBus;

    pub struct TestBus {
        pub memory: [u8; 0x10000],
    }

    impl Default for TestBus {
        fn default() -> Self {
            Self { memory: [0; 0x10000] }
        }
    }

    impl InterruptBus for TestBus {}

    impl BusIO for TestBus {
        fn read_byte(&self, address: u16) -> u8 {
            self.memory[address as usize]
        }

        fn write_byte(&mut self, address: u16, byte: u8) {
            self.memory[address as usize] = byte;
        }

        fn write_internal_byte(&mut self, address: u16, byte: u8) {
            self.memory[address as usize] = byte;
        }

        fn read_word(&self, address: u16) -> u16 {
            (self.memory[address as usize] as u16)  // LSB first
                | (self.memory[address as usize + 1] as u16) << 8 // MSB second
        }

        fn write_word(&mut self, address: u16, word: u16) {
            self.memory[address as usize] = word as u8;
            self.memory[address as usize + 1] = (word >> 8) as u8;
        }
    }

    impl CpuBus for TestBus {}

    #[test]
    fn test_bus() {
        let mut bus = TestBus::default();

        // Test byte operations
        bus.write_byte(0x1234, 0x42);
        assert_eq!(bus.read_byte(0x1234), 0x42);

        // Test word operations
        bus.write_word(0x4321, 0xABCD);
        assert_eq!(bus.read_word(0x4321), 0xABCD);
    }
}
