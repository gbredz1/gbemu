use bitflags::bitflags;

// Define bitflags for LCD Control Register ($FF40)
bitflags! {
    /// LCD Control Register (LCDC) at address $FF40
    /// Controls basic LCD operation and display settings
    pub struct LcdControl: u8 {
        /// Enable or disable the LCD & PPU (0=Off, 1=On)
        const ENABLE    = 0b1000_0000;
        /// Window Tile Map area (0=9800-9BFF, 1=9C00-9FFF)
        const WINDOW_TILE_MAP = 0b0100_0000;
        /// Enable or disable the Window display
        const WINDOW_ENABLE     = 0b0010_0000;
        /// BG & Window Tile Data area (0=8800-97FF, 1=8000-8FFF)
        const BG_WINDOW_TILES   = 0b0001_0000;
        /// BG Tile Map area (0=9800-9BFF, 1=9C00-9FFF)
        const BG_TILE_MAP     = 0b0000_1000;
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

// Define additional bitflags for palette registers
bitflags! {
    /// Background Palette (BGP) at address $FF47
    /// Maps color numbers to actual shades of gray
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Palette: u8 {
        /// Color for index 3 (2 bits, shift 6)
        const COLOR_3 = 0b1100_0000;
        /// Color for index 2 (2 bits, shift 4)
        const COLOR_2 = 0b0011_0000;
        /// Color for index 1 (2 bits, shift 2)
        const COLOR_1 = 0b0000_1100;
        /// Color for index 0 (2 bits, shift 0)
        const COLOR_0 = 0b0000_0011;
    }
}

#[allow(dead_code)]
pub trait PpuBus {
    fn read_byte(&self, address: u16) -> u8;
    fn write_byte(&mut self, address: u16, value: u8);

    fn lcdc(&self) -> LcdControl {
        LcdControl::from_bits_truncate(self.read_byte(0xFF40))
    }
    fn set_lcdc(&mut self, value: LcdControl) {
        self.write_byte(0xFF40, value.bits());
    }
    fn update_lcdc(&mut self, flags: LcdControl, enabled: bool) {
        self.lcdc().set(flags, enabled);
    }
    fn stat(&self) -> LcdStatus {
        LcdStatus::from_bits_truncate(self.read_byte(0xFF41))
    }
    fn set_stat(&mut self, value: LcdStatus) {
        self.write_byte(0xFF41, value.bits());
    }
    fn update_stat(&mut self, flags: LcdStatus, enabled: bool) {
        self.stat().set(flags, enabled);
    }
    fn scy(&self) -> u8 {
        self.read_byte(0xFF42)
    }
    fn set_scy(&mut self, value: u8) {
        self.write_byte(0xFF42, value);
    }
    fn scx(&self) -> u8 {
        self.read_byte(0xFF43)
    }
    fn set_scx(&mut self, value: u8) {
        self.write_byte(0xFF43, value);
    }
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
            todo!("request stat interrupt")
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
    fn bgp(&self) -> Palette {
        Palette::from_bits_truncate(self.read_byte(0xFF47))
    }
    fn set_bgp(&mut self, value: Palette) {
        self.write_byte(0xFF47, value.bits());
    }
    fn update_bgp(&mut self, flags: Palette, enabled: bool) {
        self.bgp().set(flags, enabled);
    }
    fn obp0(&self) -> Palette {
        Palette::from_bits_truncate(self.read_byte(0xFF48))
    }
    fn set_obp0(&mut self, value: Palette) {
        self.write_byte(0xFF48, value.bits());
    }
    fn update_obp0(&mut self, flags: Palette, enabled: bool) {
        self.obp0().set(flags, enabled);
    }
    fn obp1(&self) -> Palette {
        Palette::from_bits_truncate(self.read_byte(0xFF49))
    }
    fn set_obp1(&mut self, value: Palette) {
        self.write_byte(0xFF49, value.bits());
    }
    fn update_obp1(&mut self, flags: Palette, enabled: bool) {
        self.obp1().set(flags, enabled);
    }
    fn wy(&self) -> u8 {
        self.read_byte(0xFF4A)
    }
    fn set_wy(&mut self, value: u8) {
        self.write_byte(0xFF4A, value);
    }
    fn wx(&self) -> u8 {
        self.read_byte(0xFF4B)
    }
    fn set_wx(&mut self, value: u8) {
        self.write_byte(0xFF4B, value);
    }
}
