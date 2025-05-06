use crate::bus::{InterruptBus, define_flags_accessors, define_u8_accessors};
use bitflags::bitflags;

bitflags! {
    /// OAM DMA source address
    /// Specifies the top 8 bits of the OAM DMA source addr
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TAC: u8 {
        const Enable = 0b0000_0100;
        const ClockSelect1 = 0b0000_0010;
        const ClockSelect0 = 0b0000_0001;
    }
}

pub(crate) trait TimerBus: InterruptBus {
    define_u8_accessors!(div, 0xFF04);
    define_u8_accessors!(tima, 0xFF05);
    define_u8_accessors!(tma, 0xFF06);
    define_flags_accessors!(tac, 0xFF07, TAC);
}
