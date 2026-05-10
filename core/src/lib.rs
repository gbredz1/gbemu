pub(crate) mod bus;
pub(crate) mod cartridge;
pub(crate) mod cpu;
pub(crate) mod debug;
pub(crate) mod joypad;
pub(crate) mod machine;
pub(crate) mod ppu;
pub mod serial;
mod tests;
mod timer;

pub use bus::*;
pub use cpu::{Cpu, CpuBus, Flags as CpuFlags};
pub use joypad::Button as JoypadButton;
pub use machine::Machine;
pub use timer::Timer;

pub const FRAME_DURATION: std::time::Duration = std::time::Duration::from_nanos(16_742_706); // 1/59.7275 s
pub const CYCLES_PER_FRAME: usize = 70224;

#[cfg(any(test, feature = "test-bus"))]
pub use crate::tests::bus::TestBus;
