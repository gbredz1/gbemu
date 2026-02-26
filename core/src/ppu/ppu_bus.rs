use crate::bus::{InterruptBus, define_flags_accessors, define_palette_accessors, define_u8_accessors};
use crate::ppu::mode::Mode;
use bitflags::bitflags;

// Define bitflags for LCD Control Register ($FF40)
bitflags! {
    /// LCD Control Register (LCDC) at address $FF40
    /// Controls basic LCD operation and display settings
    pub struct LcdControl: u8 {
        /// Enable or disable the LCD & PPU (0=Off, 1=On)
        const ENABLE            = 0b1000_0000;
        /// Window Tile Map area (0=9800-9BFF, 1=9C00-9FFF)
        const WINDOW_TILE_MAP   = 0b0100_0000;
        /// Enable or disable the Window display
        const WINDOW_ENABLE     = 0b0010_0000;
        /// BG & Window Tile Data area (0=8800-97FF, 1=8000-8FFF)
        const TILEDATA_AREA     = 0b0001_0000;
        /// BG Tile Map area (0=9800-9BFF, 1=9C00-9FFF)
        const TILEMAP_AREA      = 0b0000_1000;
        /// Sprite size (0=8x8, 1=8x16)
        const OBJ_SIZE          = 0b0000_0100;
        /// Enable or disable Sprite display
        const OBJ_ENABLE        = 0b0000_0010;
        /// Enable or disable Background & Window display
        const BG_WINDOW_ENABLE  = 0b0000_0001;
    }
}

// Define bitflags for LCD Status Register ($FF41)
bitflags! {
    /// LCD Status Register (STAT) at address $FF41
    /// Controls LCD interrupt sources and indicates current LCD mode
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct LcdStatus: u8 {
        /// LYC int select (Read/Write): If set, selects the LYC == LY condition for the STAT interrupt.
        const LYC_INTERRUPT     = 0b0100_0000;
        /// Mode 2 int select (Read/Write): If set, selects the Mode 2 condition for the STAT interrupt.
        const OAM_INTERRUPT     = 0b0010_0000;
        /// Mode 1 int select (Read/Write): If set, selects the Mode 1 condition for the STAT interrupt.
        const VBLANK_INTERRUPT  = 0b0001_0000;
        /// Mode 0 int select (Read/Write): If set, selects the Mode 0 condition for the STAT interrupt.
        const HBLANK_INTERRUPT  = 0b0000_1000;
        /// LYC == LY (Read-only): Set when LY contains the same value as LYC; it is constantly updated.
        const LYC_EQUAL         = 0b0000_0100;
        /// PPU mode (Read-only): Indicates the PPU’s current status. Reports 0 instead when the PPU is disabled.
        const MODE_BIT_1        = 0b0000_0010;
        const MODE_BIT_0        = 0b0000_0001;
    }
}

bitflags! {
    /// OAM DMA source address
    /// Specifies the top 8 bits of the OAM DMA source addr
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DMA: u8 {
        const DMA_7 = 0b1000_0000;
        const DMA_6 = 0b0100_0000;
        const DMA_5 = 0b0010_0000;
        const DMA_4 = 0b0001_0000;
        const DMA_3 = 0b0000_1000;
        const DMA_2 = 0b0000_0100;
        const DMA_1 = 0b0000_0010;
        const DMA_0 = 0b0000_0001;
    }
}

#[allow(dead_code)]
pub trait PpuBus: InterruptBus {
    define_flags_accessors!(lcdc, 0xFF40, LcdControl);
    define_flags_accessors!(stat, 0xFF41, LcdStatus);
    define_u8_accessors!(scy, 0xFF42);
    define_u8_accessors!(scx, 0xFF43);
    fn ly(&self) -> u8 {
        self.read_byte(0xFF44)
    }
    fn set_ly(&mut self, value: u8) {
        self.write_byte(0xFF44, value);

        // update LYC=LY flag in STAT
        let lyc = self.lyc();
        let bit = value == lyc;
        self.update_stat(LcdStatus::LYC_EQUAL, bit);

        if bit {
            //    todo!("request stat interrupt")
        }
    }
    fn lyc(&self) -> u8 {
        self.read_byte(0xFF45)
    }
    fn set_lyc(&mut self, value: u8) {
        let actual = self.lyc();
        if actual != value {
            self.write_byte(0xFF45, value);
            self.set_ly(self.ly()); // Update LYC=LY flag when LYC is modified
        }
    }
    define_flags_accessors!(dma, 0xFF46, DMA);
    define_palette_accessors!(bgp, 0xFF47);
    define_palette_accessors!(obp0, 0xFF48);
    define_palette_accessors!(obp1, 0xFF49);
    define_u8_accessors!(wy, 0xFF4A);
    define_u8_accessors!(wx, 0xFF4B);
    fn read_oam(&self, address: u16) -> u8 {
        self.read_byte(0xFE00 + address)
    }
    fn read_vram(&self, address: u16) -> u8 {
        self.read_byte(0x8000 + address)
    }
    fn read_mode(&self) -> Mode {
        match self.stat().bits() & 0x03 {
            0 => Mode::HBlank,
            1 => Mode::VBlank,
            2 => Mode::OAMScan,
            3 => Mode::PixelTransfer,
            _ => unreachable!(),
        }
    }
    fn write_mode(&mut self, mode: Mode) {
        let val = mode as u8 & 0x03;
        self.update_stat(LcdStatus::MODE_BIT_1, val & 2 > 0);
        self.update_stat(LcdStatus::MODE_BIT_0, val & 1 > 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::bus::TestBus;

    impl PpuBus for TestBus {}
}
