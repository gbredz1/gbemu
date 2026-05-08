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
    pub fn bg_priority(&self) -> bool {
        self.attributes.contains(Attributes::PRIORITY)
    }
    pub fn y(&self) -> i16 {
        self.y
    }
    pub fn tile_index(&self) -> u8 {
        self.tile_index
    }
}
