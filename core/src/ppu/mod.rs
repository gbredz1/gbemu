use crate::bus::Interrupt;
use crate::ppu::mode::Mode;
pub(crate) use crate::ppu::ppu_bus::PpuBus;
pub(crate) use crate::ppu::ppu_bus::{LcdControl, LcdStatus};

mod mode;
mod ppu_bus;

pub(crate) struct Ppu {
    // Internal status
    mode_clock: u64, // Cycle counter for current mode
    current_line_sprites: Vec<(u8, u8, u8, u8)>,

    // buffer
    pub frame_buffer: Vec<u8>,
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            mode_clock: 0,
            frame_buffer: vec![0; 160 * 144],
            current_line_sprites: Vec::with_capacity(10),
        }
    }
}

impl Ppu {
    pub fn reset(&mut self, bus: &mut impl PpuBus) {
        bus.write_mode(Mode::HBlank);
        self.mode_clock = 0;
        self.frame_buffer.fill(4);
        self.current_line_sprites.clear();

        // ly and lyc can update LCDC
        bus.set_ly(0);
        bus.set_lyc(0);

        bus.set_lcdc_u8(0x91);
        bus.set_stat_u8(0x80);
        bus.set_scy(0);
        bus.set_scx(0);
        bus.set_dma_u8(0xFF);
        bus.set_bgp(0xFC);
        bus.set_obp0(0xFF);
        bus.set_obp1(0xFF);
        bus.set_wy(0);
        bus.set_wx(0);
    }

    pub fn update(&mut self, bus: &mut impl PpuBus, cycles: u32) {
        if !bus.lcdc().contains(LcdControl::ENABLE) {
            return;
        }

        self.mode_clock += cycles as u64;
        const CYCLES_PER_LINE: u64 = 456;

        if self.mode_clock < CYCLES_PER_LINE {
            return;
        }

        self.mode_clock -= CYCLES_PER_LINE;

        let current_ly = bus.ly();
        let new_ly = (current_ly + 1) % 154;
        bus.set_ly(new_ly);

        if new_ly == bus.lyc() {
            bus.update_stat(LcdStatus::LYC_EQUAL, true);
            if bus.stat().contains(LcdStatus::LYC_INTERRUPT) {
                bus.update_interrupt_flag(Interrupt::LCD_STAT, true);
            }
        } else {
            bus.update_stat(LcdStatus::LYC_EQUAL, false);
        }

        if new_ly < 144 {
            self.render_line(bus, new_ly);
            bus.write_mode(Mode::HBlank);
        } else if new_ly == 144 {
            bus.write_mode(Mode::VBlank);
            bus.update_interrupt_flag(Interrupt::VBLANK, true);
        } else {
            bus.write_mode(Mode::VBlank);
        }
    }

    fn render_line(&mut self, bus: &impl PpuBus, line: u8) {
        if line >= 144 {
            return;
        }

        let tiledata_mode_0 = bus.lcdc().contains(LcdControl::TILEDATA_AREA);
        let tilemap = if bus.lcdc().contains(LcdControl::TILEMAP_AREA) {
            0x1C00 // at $9C00
        } else {
            0x1800 // at $9800
        };

        let y = line as u16;
        let scroll_y = bus.scy() as u16;
        let scroll_x = bus.scx() as u16;

        // Draw background
        for x in 0..160 {
            let bg_y = (y + scroll_y) % 256;
            let bg_x = (x + scroll_x) % 256;

            let tile_y = bg_y / 8;
            let tile_x = bg_x / 8;

            let py = bg_y % 8;
            let px = bg_x % 8;

            let tile_addr = tilemap + tile_x + tile_y * 32;
            let tile_value = bus.read_vram(tile_addr) as u16;

            let tile_data_addr = if tiledata_mode_0 {
                tile_value * 16
            } else if tile_value < 128 {
                0x1000 + tile_value * 16
            } else {
                0x0800 + (tile_value - 128) * 16
            };

            let line_addr = tile_data_addr + py * 2;

            //  pixel value
            let low_byte = bus.read_vram(line_addr);
            let high_byte = bus.read_vram(line_addr + 1);
            let bit_pos = 7 - px;

            // apply palette
            let color_low = (low_byte >> bit_pos) & 0x01;
            let color_high = (high_byte >> bit_pos) & 0x01;
            let color_id = (color_high << 1) | color_low;
            let color = bus.bgp_color(color_id);

            self.frame_buffer[(y * 160 + x) as usize] = color;
        }
    }

    #[allow(dead_code)]
    #[cfg(debug_assertions)]
    pub fn update_test(&mut self, bus: &mut impl PpuBus, _cycles: u32) {
        // Check if the LCD screen is enabled
        if bus.lcdc().bits() & 0x80 == 0 {
            return; // LCD disabled, do nothing
        }

        // For the simplified version, fill the buffer with a test pattern.
        self.render_test_pattern(bus);
    }
    // Function to display a simple test pattern
    #[cfg(debug_assertions)]
    fn render_test_pattern(&mut self, _bus: &impl PpuBus) {
        for y in 0..144 {
            for x in 0..160 {
                // Checkerboard pattern for testing
                let color = if (x / 8 + y / 8) % 2 == 0 { 3 } else { 1 };
                self.frame_buffer[y * 160 + x] = color;
            }
        }

        // border 1 px
        for x in 0..160 {
            self.frame_buffer[x] = 0; // Haut
            self.frame_buffer[143 * 160 + x] = 0; // Bas
        }
        for y in 0..144 {
            self.frame_buffer[y * 160] = 0; // Gauche
            self.frame_buffer[y * 160 + 159] = 0; // Droite
        }
    }
}
