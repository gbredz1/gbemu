use crate::bus::{InterruptBus, define_flags_accessors, define_u8_accessors};
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SC: u8 {
        const Enable = 0b1000_0000;
        const ClockSpeed = 0b0000_0010;
        const ClockSelect = 0b0000_0001;
    }
}

#[allow(dead_code)]
pub trait SerialBus: InterruptBus {
    define_u8_accessors!(sb, 0xFF01);
    define_flags_accessors!(sc, 0xFF02, SC);
}
