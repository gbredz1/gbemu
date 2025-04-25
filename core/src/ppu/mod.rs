use crate::bus::Interrupt;
use crate::ppu::mode::Mode;
pub(crate) use crate::ppu::ppu_bus::PpuBus;
use crate::ppu::ppu_bus::{LcdControl, LcdStatus, Palette};

mod mode;
mod ppu_bus;

pub(crate) struct Ppu {
    // Internal status
    mode: Mode,      // Mode (0-3)
    mode_clock: u32, // Cycle counter for current mode
    current_line_sprites: Vec<(u8, u8, u8, u8)>,

    // buffer
    pub frame_buffer: Vec<u8>,
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            mode: Mode::HBlank,
            mode_clock: 0,
            frame_buffer: vec![0; 160 * 144],
            current_line_sprites: Vec::with_capacity(10),
        }
    }
}

impl Ppu {
    pub fn reset(&mut self, bus: &mut impl PpuBus) {
        self.mode = Mode::HBlank;
        self.mode_clock = 0;
        self.frame_buffer.fill(4);
        self.current_line_sprites.clear();

        bus.set_ly(0);
        bus.update_stat(
            // By default, we're in Mode 0 (HBlank)
            LcdStatus::MODE_BIT_0
                | LcdStatus::MODE_BIT_1
                // Reset LYC interrupt flags
                | LcdStatus::LYC_INTERRUPT
                | LcdStatus::HBLANK_INTERRUPT
                | LcdStatus::VBLANK_INTERRUPT
                | LcdStatus::OAM_INTERRUPT
                // Reset LYC flag
                | LcdStatus::LYC_EQUAL,
            false,
        );

        // Reset scroll registers
        bus.set_scy(0);
        bus.set_scx(0);

        // Reset window position
        bus.set_wy(0);
        bus.set_wx(0);

        // Reset palettes to their default values
        bus.set_bgp(Palette::from_bits_truncate(0xFC)); // Default value for BGP (11111100) - colors 3, 2, 1, 0
        bus.set_obp0(Palette::from_bits_truncate(0xFF)); // Default value for OBP0
        bus.set_obp1(Palette::from_bits_truncate(0xFF)); // Default value for OBP1
    }

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
            bus.request_interrupt(Interrupt::VBLANK);
        } else {
            // Go back to OAM scan mode
            self.write_ppu_mode(bus, Mode::OAMScan);

            // Trigger LCD STAT interrupt if OAM is enabled in STAT
            if bus.stat().contains(LcdStatus::OAM_INTERRUPT) {
                bus.request_interrupt(Interrupt::LCD_STAT);
            }
        }
    }
    /// V-Blank mode
    fn handle_mode1(&mut self, bus: &mut impl PpuBus) {
        // VBlank lasts 456 cycles per line, for 10 lines (145-154)
        if self.mode_clock < 456 {
            return;
        }
        self.mode_clock = 0;

        // Increment LY
        let line = bus.ly() + 1;
        bus.set_ly(line);

        // Check if we've reached the end of VBlank (line 154)
        if line == 154 {
            // Reset LY to 0 and go back to OAM scan mode for the next frame
            bus.set_ly(0);
            self.write_ppu_mode(bus, Mode::OAMScan);

            // Trigger LCD STAT interrupt if OAM is enabled in STAT
            if bus.stat().contains(LcdStatus::OAM_INTERRUPT) {
                bus.request_interrupt(Interrupt::LCD_STAT);
            }
        }
    }
    /// OAM-Scan mode
    fn handle_mode2(&mut self, bus: &mut impl PpuBus) {
        // OAM Scan mode lasts for 80 cycles
        if self.mode_clock < 80 {
            return;
        }
        self.mode_clock = 0;

        // Collecte des sprites visibles sur la ligne actuelle
        let current_line = bus.ly();
        let mut visible_sprites = Vec::with_capacity(10); // Max 10 sprites per line

        // Parcourir les 40 sprites dans l'OAM
        for sprite_idx in 0..40 {
            // Lire les données du sprite depuis l'OAM
            let sprite_y = bus.read_oam(sprite_idx * 4);
            let sprite_x = bus.read_oam(sprite_idx * 4 + 1);
            let tile_idx = bus.read_oam(sprite_idx * 4 + 2);
            let attributes = bus.read_oam(sprite_idx * 4 + 3);

            // Déterminer la hauteur du sprite (8x8 ou 8x16)
            let sprite_height = if bus.lcdc().contains(LcdControl::OBJ_SIZE) {
                16
            } else {
                8
            };

            // Vérifier si le sprite est visible sur la ligne actuelle
            // Position Y est décalée de 16 pixels dans les spécifications GameBoy
            let sprite_line = current_line as i32 - (sprite_y as i32 - 16);
            if sprite_line >= 0 && sprite_line < sprite_height as i32 {
                // Le sprite est visible sur cette ligne
                visible_sprites.push((sprite_idx as u8, sprite_x, tile_idx, attributes));

                // Limitation à 10 sprites par ligne comme sur le hardware
                if visible_sprites.len() >= 10 {
                    break;
                }
            }
        }

        // Trier les sprites par priorité (position X, plus petit d'abord comme sur le hardware)
        visible_sprites.sort_by_key(|&(_, x, _, _)| x);

        // Stocker les sprites pour utilisation dans le mode 3 (Pixel Transfer)
        self.current_line_sprites = visible_sprites;

        // After OAM scan is complete, switch to Pixel Transfer mode
        self.write_ppu_mode(bus, Mode::PixelTransfer);

        // After OAM scan is complete, switch to Pixel Transfer mode
        self.write_ppu_mode(bus, Mode::PixelTransfer);

        // Trigger LCD STAT interrupt if configured for Mode::PixelTransfer mode
        if bus.stat().contains(LcdStatus::OAM_INTERRUPT) {
            bus.request_interrupt(Interrupt::LCD_STAT);
        }
    }
    /// Pixel transfer mode
    fn handle_mode3(&mut self, bus: &mut impl PpuBus) {
        // Pixel Transfer mode lasts for 172 cycles
        if self.mode_clock < 172 {
            return;
        }
        self.mode_clock = 0;

        // Get the current line
        let line = bus.ly();

        // Only render if LCD is enabled
        if bus.lcdc().contains(LcdControl::ENABLE) {
            self.render_scanline(bus, line);
        }

        // Switch to H-Blank mode after pixel transfer is complete
        self.write_ppu_mode(bus, Mode::HBlank);

        // Trigger LCD STAT interrupt if H-Blank is enabled in STAT
        if bus.stat().contains(LcdStatus::HBLANK_INTERRUPT) {
            bus.request_interrupt(Interrupt::LCD_STAT);
        }
    }

    // todo move to ppu_bus
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

    /// Renders a single scanline to the frame buffer
    fn render_scanline(&mut self, bus: &mut impl PpuBus, line: u8) {
        let line_usize = line as usize;
        let start_index = line_usize * 160; // 160 pixels per line

        // Get the background and window tiles to render
        // Background
        if bus.lcdc().contains(LcdControl::BG_WINDOW_ENABLE) {
            //   self.render_background(bus, line, start_index);
        } else {
            // If the background is disabled, fill with "white" (color 0)
            for x in 0..160 {
                self.frame_buffer[start_index + x] = 0;
            }
        }

        // Window (if enabled)
        if bus.lcdc().contains(LcdControl::WINDOW_ENABLE) {
            self.render_window(bus, line, start_index);
        }

        // Sprites (if enabled)
        if bus.lcdc().contains(LcdControl::OBJ_ENABLE) {
            //  self.render_sprites(bus, line, start_index);
        }
    }
    fn render_background(&mut self, bus: &impl PpuBus, line: u8, start_index: usize) {
        // Get the background tile map address (0x9800 or 0x9C00)
        let bg_tile_map = if bus.lcdc().contains(LcdControl::BG_TILE_MAP) {
            0x1C00
        } else {
            0x1800
        };

        // Get tile data address (0x8000 or 0x8800)
        let using_8000_addressing = bus.lcdc().contains(LcdControl::BG_TILE_MAP);

        // Get scroll positions
        let scroll_y = bus.scy();
        let scroll_x = bus.scx();

        // Calculate which row of tiles to use based on the current line and scroll_y
        let y_pos = scroll_y.wrapping_add(line);
        let tile_row = (y_pos / 8) as usize;

        // Get which line of the tile is being rendered
        let tile_y = (y_pos % 8) as usize;

        // Render all 160 pixels in the current line
        for x in 0..160u8 {
            // Get the current x position with scroll applied
            let x_pos = scroll_x.wrapping_add(x);

            // Which tile column is this pixel in
            let tile_col = (x_pos / 8) as usize;

            // Get the tile number from the background map
            let tile_map_address = bg_tile_map + (tile_row * 32) + tile_col;
            let tile_number = bus.read_vram(tile_map_address as u16) as i8;

            // Get the actual tile data
            let tile_data_address: u16 = if using_8000_addressing {
                // 0x8000 addressing - tile number is unsigned
                (tile_number * 16) as u16
            } else {
                // 0x8800 addressing - tile number is signed
                0x1000 + (tile_number * 16) as u16
            };

            // Get the specific line of tile data (each line is 2 bytes)
            let line_data_address = tile_data_address + (tile_y as u16 * 2);
            let tile_data_low = bus.read_vram(line_data_address);
            let tile_data_high = bus.read_vram(line_data_address + 1);

            // Get the specific pixel in the tile line
            let tile_x = (x_pos % 8) as usize;
            let bit_position = 7 - tile_x; // Pixels are stored with MSB on the left

            // Combine the two bits to get the color
            let color_low = (tile_data_low >> bit_position) & 0x01;
            let color_high = ((tile_data_high >> bit_position) & 0x01) << 1;
            let color_number = color_high | color_low;

            // Map the color through the background palette register (BGP)
            let bgp = bus.bgp().bits();
            let color = (bgp >> (color_number * 2)) & 0x03;

            // Set the pixel in the frame buffer (0-3 for different shades of green)
            self.frame_buffer[start_index + x as usize] = color;
        }
    }
    fn render_window(&mut self, bus: &impl PpuBus, line: u8, start_index: usize) {
        // Get the window position
        let window_y = bus.wy();
        let window_x = bus.wx().wrapping_sub(7); // WX is offset by 7 in hardware

        // Check if the window is visible on this line
        if line < window_y {
            return;
        }

        // Get the window tile map address (0x9800 or 0x9C00)
        let window_tile_map = if bus.lcdc().contains(LcdControl::WINDOW_TILE_MAP) {
            0x1C00
        } else {
            0x1800
        };

        // Get tile data address (0x8000 or 0x8800)
        let using_8000_addressing = bus.lcdc().contains(LcdControl::WINDOW_TILE_MAP);

        // Calculate which row of tiles to use based on the current line
        let window_line = line - window_y;
        let tile_row = (window_line / 8) as usize;

        // Get which line of the tile is being rendered
        let tile_y = (window_line % 8) as usize;

        // Render window pixels for the current line
        for x in 0..160u8 {
            // Skip pixels before the window starts
            if x < window_x {
                continue;
            }

            // Calculate the position within the window
            let window_x = x - window_x;

            // Which tile column is this pixel in
            let tile_col = (window_x / 8) as usize;

            // Get the tile number from the window map
            let tile_map_address = window_tile_map + (tile_row * 32) + tile_col;
            let tile_number = bus.read_vram(tile_map_address as u16) as i8;

            // Get the actual tile data
            let tile_data_address = if using_8000_addressing {
                // 0x8000 addressing - tile number is unsigned
                tile_number as u16 * 16
            } else {
                // 0x8800 addressing - tile number is signed
                0x1000 + (tile_number * 16) as u16
            };

            // Get the specific line of tile data (each line is 2 bytes)
            let line_data_address = tile_data_address + (tile_y as u16 * 2);
            let tile_data_low = bus.read_vram(line_data_address);
            let tile_data_high = bus.read_vram(line_data_address + 1);

            // Get the specific pixel in the tile line
            let tile_x = (window_x % 8) as usize;
            let bit_position = 7 - tile_x; // Pixels are stored with MSB on the left

            // Combine the two bits to get the color
            let color_low = (tile_data_low >> bit_position) & 0x01;
            let color_high = ((tile_data_high >> bit_position) & 0x01) << 1;
            let color_number = color_high | color_low;

            // Map the color through the background palette register (BGP)
            let bgp = bus.bgp().bits();
            let color = (bgp >> (color_number * 2)) & 0x03;

            // Set the pixel in the frame buffer (0-3 for different shades of green)
            self.frame_buffer[start_index + x as usize] = color;
        }
    }
    fn render_sprites(&mut self, bus: &impl PpuBus, line: u8, start_index: usize) {
        let sprite_height = if bus.lcdc().contains(LcdControl::OBJ_SIZE) {
            16 // 8x16 sprites
        } else {
            8 // 8x8 sprites
        };

        // Iterate through all sprites found during OAM scan for this line
        for &(sprite_idx, sprite_x, tile_idx, attributes) in &self.current_line_sprites {
            // Skip if sprite is off-screen (X=0 means off left edge of screen)
            if sprite_x == 0 || sprite_x >= 168 {
                continue;
            }

            // Get the actual X position on screen (X coordinate is offset by 8 in hardware)
            let x_pos = sprite_x.wrapping_sub(8);

            // Get the Y position from OAM
            let sprite_y = bus.read_oam((sprite_idx as u16) * 4);
            let y_pos = sprite_y.wrapping_sub(16); // Y coordinate is offset by 16 in hardware

            // Calculate which line of the sprite we're rendering
            let mut sprite_line = line.wrapping_sub(y_pos);

            // Handle Y flip
            if attributes & 0x40 != 0 {
                sprite_line = (sprite_height - 1) - sprite_line;
            }

            // For 8x16 sprites, handle the tile selection differently
            let tile_number = if sprite_height == 16 {
                // For 8x16 mode, the lower bit of the tile number is ignored
                (tile_idx & 0xFE) + if sprite_line >= 8 { 1 } else { 0 }
            } else {
                tile_idx
            };

            // Calculate the address of the tile data
            let tile_address = tile_number as u16 * 16;

            // Adjust sprite_line for the second tile in 8x16 mode
            let adjusted_sprite_line = if sprite_height == 16 && sprite_line >= 8 {
                sprite_line - 8
            } else {
                sprite_line
            };

            // Get the specific line of tile data (each line is 2 bytes)
            let line_data_address = tile_address + (adjusted_sprite_line as u16 * 2);
            let tile_data_low = bus.read_vram(line_data_address);
            let tile_data_high = bus.read_vram(line_data_address + 1);

            // Get the palette
            let palette = if attributes & 0x10 != 0 {
                bus.obp1() // OBP1
            } else {
                bus.obp0() // OBP0
            };

            // Render each pixel of the sprite
            for x in 0..8u8 {
                // Skip if this pixel is off-screen
                if x_pos.wrapping_add(x) >= 160 {
                    continue;
                }

                // Handle X flip
                let bit_position = if attributes & 0x20 != 0 { x } else { 7 - x };

                // Combine the two bits to get the color number
                let color_low = (tile_data_low >> bit_position) & 0x01;
                let color_high = ((tile_data_high >> bit_position) & 0x01) << 1;
                let color_number = color_high | color_low;

                // Skip transparent pixels (color 0)
                if color_number == 0 {
                    continue;
                }

                // Calculate screen position
                let screen_x = x_pos.wrapping_add(x) as usize;

                // Check background priority (if sprite should appear behind background colors 1-3)
                let bg_priority = (attributes & 0x80) != 0;
                if bg_priority {
                    // Get the background pixel at this position
                    let bg_pixel = self.frame_buffer[start_index + screen_x];
                    // Skip if the background pixel is not color 0 (transparent)
                    if bg_pixel != 0 {
                        continue;
                    }
                }

                // Map the color through the sprite palette register (OBP0 or OBP1)
                let color = (palette.bits() >> (color_number * 2)) & 0x03;

                // Set the pixel in the frame buffer
                self.frame_buffer[start_index + screen_x] = color;
            }
        }
    }
}
