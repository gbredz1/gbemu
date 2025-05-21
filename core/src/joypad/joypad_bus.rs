use crate::bus::{InterruptBus, define_flags_accessors};
use bitflags::bitflags;

bitflags! {
    #[derive(Default, Clone, Copy, Eq, PartialEq)]
    pub struct P1JOYP: u8 {
        // Select
        const SELECT_BUTTONS = 0b0010_0000;
        const SELECT_DPAD    = 0b0001_0000;
        // D-pad
        const RIGHT  = 0b0000_0001;
        const LEFT   = 0b0000_0010;
        const UP     = 0b0000_0100;
        const DOWN   = 0b0000_1000;
        // Boutons
        const A      = 0b0000_0001;
        const B      = 0b0000_0010;
        const SELECT = 0b0000_0100;
        const START  = 0b0000_1000;
    }
}

#[allow(dead_code)]
pub(crate) trait JoypadBus: InterruptBus {
    define_flags_accessors!(p1joyp, 0xFF00, P1JOYP);
}
