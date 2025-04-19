use crate::ppu::mode::Mode;
pub(crate) use crate::ppu::ppu_bus::PpuBus;
use crate::ppu::ppu_bus::{LcdControl, LcdStatus};

mod mode;
mod ppu_bus;

pub(crate) struct Ppu {
    // Internal status
    mode: Mode,      // Mode (0-3)
    mode_clock: u32, // Cycle counter for current mode

    // buffer
    frame_buffer: [u8; 160 * 144 * 4],
    frame_complete: bool,
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            mode: Mode::HBlank,
            mode_clock: 0,
            frame_buffer: [0; 160 * 144 * 4],
            frame_complete: false,
        }
    }
}

impl Ppu {
    pub fn update(&mut self, bus: &mut impl PpuBus, cycles: u32) {
        let lcdc = bus.lcdc();
        if !lcdc.contains(LcdControl::ENABLE) {
            self.mode_clock += cycles;
            return;
        }

        for _ in 0..cycles {
            self.mode_clock += 1;
            self.mode = self.read_ppu_mode(bus);
            match self.mode {
                Mode::HBlank => self.handle_mode0(bus),
                Mode::VBlank => self.handle_mode1(bus),
                Mode::OAMScan => self.handle_mode2(bus),
                Mode::PixelTransfer => self.handle_mode3(bus),
            }
        }
    }

    /// H-Blank mode
    fn handle_mode0(&mut self, bus: &mut impl PpuBus) {
        if self.mode_clock < 204 {
            return;
        }
        self.mode_clock = 0;

        // inc LY
        let line = bus.ly() + 1;
        bus.set_ly(line);

        // Check if we should enter V-Blank
        if line == 144 {
            self.write_ppu_mode(bus, Mode::VBlank);
            bus.set_stat(LcdStatus::VBLANK_INTERRUPT);
        } else {
            // Go back to OAM scan mode
            self.write_ppu_mode(bus, Mode::OAMScan);
        }
    }
    fn handle_mode1(&mut self, bus: &mut impl PpuBus) {}
    fn handle_mode2(&mut self, bus: &mut impl PpuBus) {}
    fn handle_mode3(&mut self, bus: &mut impl PpuBus) {}

    fn read_ppu_mode(&self, bus: &impl PpuBus) -> Mode {
        match bus.stat().bits() & 0x03 {
            0 => Mode::HBlank,
            1 => Mode::VBlank,
            2 => Mode::OAMScan,
            3 => Mode::PixelTransfer,
            _ => unreachable!(),
        }
    }
    fn write_ppu_mode(&self, bus: &mut impl PpuBus, mode: Mode) {
        let val = mode as u8 & 0x03;
        bus.update_stat(LcdStatus::MODE_BIT_1, val & 2 > 0);
        bus.update_stat(LcdStatus::MODE_BIT_0, val & 1 > 0);
    }
}
