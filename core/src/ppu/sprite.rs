use crate::ppu::LCD_WIDTH;
use bitflags::bitflags;

#[derive(Debug)]
pub struct Sprite {
    x: i16,
    y: i16,
    tile_index: u8,
    attributes: Attributes,
}

bitflags! {
    #[derive(Debug)]
    pub struct Attributes : u8 {
        const PRIORITY = 0b1000_0000;
        const Y_FLIP = 0b0100_0000;
        const X_FLIP = 0b0010_0000;
        const DMG_PALETTE = 0b0001_0000; // [CGB Mode Only]
        // const BANK = 0b0000_1000; // [CGB Mode Only]
        // const CGB_PALETTE_2 = 0b0010_0100; // [CGB Mode Only]
        // const CGB_PALETTE_1 = 0b0010_0010; // [CGB Mode Only]
        // const CGB_PALETTE_0 = 0b0010_0001; // [CGB Mode Only]
    }
}

impl Sprite {
    pub fn from(bytes: [u8; 4]) -> Self {
        Self {
            x: (bytes[1] as i16) - 8,
            y: (bytes[0] as i16) - 16,
            tile_index: bytes[2],
            attributes: Attributes::from_bits_truncate(bytes[3]),
        }
    }

    pub fn x(&self) -> i16 {
        self.x
    }

    pub fn has_x_flip(&self) -> bool {
        self.attributes.contains(Attributes::X_FLIP)
    }
    pub fn has_y_flip(&self) -> bool {
        self.attributes.contains(Attributes::Y_FLIP)
    }
    pub fn palette(&self) -> bool {
        self.attributes.contains(Attributes::DMG_PALETTE)
    }
    pub fn is_visible_at_line(&self, line: u8, double_height: bool) -> bool {
        let line = line as i16;
        let height = if double_height { 16 } else { 8 };
        let width: i16 = 8; // always 8 pixels wide for sprites

        (line >= self.y && line < self.y + height) && (self.x < LCD_WIDTH as i16 && self.x + width > 0)
    }

    /// Calculates the address for the current line of a sprite tile.
    ///
    /// # Arguments
    /// * `line` - The current line number
    /// * `double_height` - true if sprite is in 8x16 mode (tall sprites), false for 8x8 mode
    ///
    /// # Returns
    /// The address containing the tile data for the current line
    ///
    /// Takes Y-flipping into account and handles both 8x8 and 8x16 sprite modes.
    pub fn get_tile_address(&self, line: u8, double_height: bool) -> u16 {
        let line = line as i16;

        // determine the line to show
        let mut line = line.saturating_sub(self.y) as u16;
        if self.has_y_flip() {
            line = if double_height { 15 } else { 7 } - line;
        }

        let index = if !double_height {
            self.tile_index
        } else {
            let idx = self.tile_index & 0xFE;
            if line < 8 { idx } else { idx + 1 }
        } as u16;

        let offset = (line % 8) * 2;

        (index << 4) + offset
    }
}
