use crate::bus::Interrupt;
use crate::ppu::mode::Mode;
pub(crate) use crate::ppu::ppu_bus::PpuBus;
pub(crate) use crate::ppu::ppu_bus::{LcdControl, LcdStatus};
use crate::ppu::sprite::Sprite;

mod mode;
mod ppu_bus;
mod sprite;

const LCD_WIDTH: u8 = 160;
const LCD_HEIGHT: u8 = 144;

pub(crate) struct Ppu {
    // Internal status
    mode_clock: u64, // Cycle counter for current mode
    sprites_visibles_on_current_line: Vec<Sprite>,

    // buffer
    pub frame_buffer: Vec<u8>,
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            mode_clock: 0,
            frame_buffer: vec![0; LCD_WIDTH as usize * LCD_HEIGHT as usize],
            sprites_visibles_on_current_line: Vec::with_capacity(10),
        }
    }
}

impl Ppu {
    pub fn reset(&mut self, bus: &mut impl PpuBus) {
        bus.write_mode(Mode::HBlank);
        self.mode_clock = 0;
        self.frame_buffer.fill(33);

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

        for addr in 0xFE00..0xFEA0 {
            bus.write_internal_byte(addr, 0);
        }
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
        let new_ly = current_ly.wrapping_add(1) % 154;
        bus.set_ly(new_ly);

        if new_ly == bus.lyc() {
            bus.update_stat(LcdStatus::LYC_EQUAL, true);
            if bus.stat().contains(LcdStatus::LYC_INTERRUPT) {
                bus.update_interrupt_flag(Interrupt::LCD_STAT, true);
            }
        } else {
            bus.update_stat(LcdStatus::LYC_EQUAL, false);
        }

        if new_ly < LCD_HEIGHT {
            self.render_line(bus, new_ly);
            bus.write_mode(Mode::HBlank);
        } else if new_ly == LCD_HEIGHT {
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

        if bus.lcdc().contains(LcdControl::BG_WINDOW_ENABLE) {
            self.render_background_line(bus, line);
        }

        if bus.lcdc().contains(LcdControl::OBJ_ENABLE) {
            let double_height = bus.lcdc().contains(LcdControl::OBJ_SIZE);
            self.update_visibles_sprites(bus, line, double_height);
            self.render_sprites_line(bus, line, double_height);
        }
    }

    fn render_background_line(&mut self, bus: &impl PpuBus, line: u8) {
        let tilemap = if bus.lcdc().contains(LcdControl::TILEMAP_AREA) {
            0x1C00 // at $9C00
        } else {
            0x1800 // at $9800
        };

        let y = line as u16;
        let scroll_y = bus.scy() as u16;
        let scroll_x = bus.scx() as u16;

        // Draw background
        for x in 0..LCD_WIDTH as u16 {
            let bg_y = (y + scroll_y) % 256;
            let bg_x = (x + scroll_x) % 256;

            let tile_y = bg_y / 8;
            let tile_x = bg_x / 8;

            let py = bg_y % 8;
            let px = bg_x % 8;

            let tile_addr = tilemap + tile_x + tile_y * 32;
            let tile_value = bus.read_vram(tile_addr) as u16;

            let tile_data_addr = if bus.lcdc().contains(LcdControl::TILEDATA_AREA) {
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

            self.frame_buffer[(y * LCD_WIDTH as u16 + x) as usize] = color;
        }
    }

    fn update_visibles_sprites(&mut self, bus: &impl PpuBus, line: u8, double_height: bool) {
        self.sprites_visibles_on_current_line.clear();

        // look at all sprites in the OAM (40 sprites max)
        for sprite_idx in (0..40 * 4).step_by(4) {
            let bytes = [
                bus.read_oam(sprite_idx),
                bus.read_oam(sprite_idx + 1),
                bus.read_oam(sprite_idx + 2),
                bus.read_oam(sprite_idx + 3),
            ];
            let sprite = Sprite::from(bytes);

            if sprite.is_visible_at_line(line, double_height) {
                self.sprites_visibles_on_current_line.push(sprite);

                if self.sprites_visibles_on_current_line.len() >= 10 {
                    break;
                }
            }
        }

        // Sort by X coordinate in descending order
        self.sprites_visibles_on_current_line
            .sort_by_key(|s| std::cmp::Reverse(s.x()));
    }

    fn render_sprites_line(&mut self, bus: &impl PpuBus, line: u8, double_height: bool) {
        for sprite in &self.sprites_visibles_on_current_line {
            let tile_addr = sprite.get_tile_address(line, double_height);

            // draw 8 pixels of the sprite
            for px in 0..8 {
                let x = (sprite.x() as usize).wrapping_add(px);
                if x >= LCD_WIDTH as usize {
                    // out of screen
                    continue;
                }

                //  pixel value
                let low_byte = bus.read_vram(tile_addr);
                let high_byte = bus.read_vram(tile_addr + 1);
                let bit_pos = if sprite.has_x_flip() { px } else { 7 - px };

                // apply palette
                let color_low = (low_byte >> bit_pos) & 0x01;
                let color_high = (high_byte >> bit_pos) & 0x01;
                let color_id = (color_high << 1) | color_low;

                // color_id == 0 means transparent for sprites
                if color_id == 0 {
                    continue;
                }

                // todo handle priority sprite/background => sprite.priority()

                // retrieve the color from the palette
                let color = if sprite.palette() {
                    bus.obp1_color(color_id)
                } else {
                    bus.obp0_color(color_id)
                };

                self.frame_buffer[line as usize * LCD_WIDTH as usize + x] = color;
            }
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
