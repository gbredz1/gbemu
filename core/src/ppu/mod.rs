use crate::bus::Interrupt;
use crate::ppu::mode::Mode;
pub(crate) use crate::ppu::ppu_bus::PpuBus;
use crate::ppu::ppu_bus::{LcdControl, LcdStatus};

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
                visible_sprites.push((sprite_idx as u8 , sprite_x, tile_idx, attributes));

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
        //todo
    }

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
