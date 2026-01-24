use crate::cpu::CpuBus;
use crate::ppu::PpuBus;
use bitflags::bitflags;
use log::{debug, error};
use std::default::Default;
use std::fs::File;
use std::io::Read;
use std::path::Path;

bitflags! {
    pub struct Interrupt: u8 {
        const VBLANK = 0b0000_0001;  // Bit 0
        const LCD_STAT = 0b0000_0010; // Bit 1
        const TIMER = 0b0000_0100;   // Bit 2
        const SERIAL = 0b0000_1000;  // Bit 3
        const JOYPAD = 0b0001_0000;  // Bit 4
    }
}

macro_rules! define_flags_accessors {
    ($name:ident, $addr:expr, $type:ty) => {
        fn $name(&self) -> $type {
            <$type>::from_bits_truncate(self.read_byte($addr))
        }

        paste::paste! {
            fn [<set_ $name>](&mut self, flags: $type) {
                let value = self.read_byte($addr) | flags.bits();
                self.write_byte($addr, value);
            }
            fn [<clear_ $name>](&mut self, flags: $type) {
                let value = self.read_byte($addr) & !flags.bits();
                self.write_byte($addr, value);
            }
            fn [<update_ $name>](&mut self, flags: $type, enabled: bool) {
                if enabled {
                    self.[<set_ $name>](flags);
                } else {
                    self.[<clear_ $name>](flags);
                }
            }
            fn [<toggle_ $name>](&mut self, flags: $type) {
                let value = self.read_byte($addr) ^ flags.bits();
                self.write_byte($addr, value);
            }
            fn [<set_ $name:lower _u8>](&mut self, value: u8) {
                self.write_byte($addr, value);
            }
        }
    };
}
pub(crate) use define_flags_accessors;
macro_rules! define_u8_accessors {
    ($name:ident, $addr:expr) => {
        fn $name(&self) -> u8 {
            self.read_byte($addr)
        }

        paste::paste! {
            fn [<set_ $name>](&mut self, value: u8) {
                self.write_byte($addr, value);
            }
        }
    };
}
pub(crate) use define_u8_accessors;
macro_rules! define_palette_accessors {
    ($name:ident, $addr:expr) => {
        fn $name(&self) -> u8 {
            self.read_byte($addr)
        }
        paste::paste! {
            fn [<$name:lower _color>](&self, color_id: u8) -> u8 {
                self.$name() >> (color_id * 2) & 0x03
            }
            fn [<set_ $name>](&mut self, value: u8) {
                self.write_byte($addr, value);
            }
        }
    };
}
use crate::cartridge::Cartridge;
use crate::joypad::joypad_bus::JoypadBus;
use crate::timer::timer_bus::TimerBus;
pub(crate) use define_palette_accessors;

pub struct MemorySystem {
    boot_rom: [u8; 0x100],
    boot_rom_enabled: bool,
    boot_rom_loaded: bool,

    vram: [u8; 0x2_000],
    wram0: [u8; 0x1_000],
    wram1: [u8; 0x1_000],
    oam: [u8; 0x100],
    io_regs: [u8; 0x80],
    hram: [u8; 0xFF],
    interrupts: u8,
    cartridge: Cartridge,
}

impl MemorySystem {
    pub fn reset(&mut self) {
        // Clear VRAM
        self.vram.fill(0);
        self.boot_rom_enabled = self.boot_rom_loaded;
    }
    pub(crate) fn cartridge(&self) -> &Cartridge {
        &self.cartridge
    }
}

impl Default for MemorySystem {
    fn default() -> Self {
        Self {
            boot_rom_enabled: false,
            boot_rom_loaded: false,
            boot_rom: [0; 0x100],
            vram: [0; 0x2_000],  // $8000..$9FFF
            wram0: [0; 0x1_000], // $C000..$CFFF
            wram1: [0; 0x1_000], // $D000..$DFFF
            oam: [0; 0x100],     // $FE00..$FE9F
            io_regs: [0; 0x80],  // $FF00..$FF7F
            hram: [0; 0xFF],     // $FF80..$FFFE
            interrupts: 0u8,     // $FFFF
            cartridge: Cartridge::empty(),
        }
    }
}

impl MemorySystem {
    pub fn load_boot_rom(&mut self) -> Result<(), std::io::Error> {
        self.boot_rom_enabled = true;
        self.boot_rom_loaded = true;

        let mut boot_file = File::open("roms/dmg.bin")?;
        boot_file.read_exact(&mut self.boot_rom)?;

        Ok(())
    }

    pub fn load_cartridge<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        self.cartridge = Cartridge::load_from_path(path)?;
        Ok(())
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        if self.boot_rom_enabled && address < 0x100 {
            unsafe { *self.boot_rom.get_unchecked(address as usize) }
        } else {
            match address {
                0x0000..=0x3FFF => self.cartridge.read_byte(address), // ROM BANK 00
                0x4000..=0x7FFF => self.cartridge.read_byte(address), // ROM BANK 01-NN
                0x8000..=0x9FFF => self.vram[address as usize - 0x8000], // VRAM
                0xA000..=0xBFFF => self.cartridge.read_byte(address), // External RAM
                0xC000..=0xCFFF => self.wram0[address as usize - 0xC000], // WRAM 0
                0xD000..=0xDFFF => self.wram1[address as usize - 0xD000], // WRAM 1
                0xE000..=0xEFFF => self.wram0[address as usize - 0xE000], // ECHO -> WRAM 0
                0xF000..=0xFDFF => self.wram1[address as usize - 0xF000], // ECHO -> WRAM 1
                0xFE00..=0xFE9F => self.oam[address as usize - 0xFE00], // OAM
                0xFEA0..=0xFEFF => 0xFF,                              // Not usable
                0xFF00..=0xFF7F => self.io_regs[address as usize - 0xFF00], // IO regs
                0xFF80..=0xFFFE => self.hram[address as usize - 0xFF80], // HRAM
                0xFFFF => self.interrupts,                            // Interrupts
            }
        }
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        if address == 0xFF04 {
            // TIMER DIV -> write = reset
            self.write_internal_byte(address, 0x00);
            return;
        }

        if address == 0xFF46 {
            // DMA transfer
            let src_addr = (byte as u16) << 8;
            for i in 0..0xA0 {
                let data = self.read_byte(src_addr + i);
                self.write_internal_byte(0xFE00 + i, data);
            }

            return;
        }

        if self.boot_rom_enabled && address < 0x100 {
            error!("Writing to boot rom is not allowed");
        } else {
            self.write_internal_byte(address, byte);

            if self.boot_rom_enabled && address == 0xFF50 {
                self.boot_rom_enabled = false;
                debug!("Boot rom disabled (${byte:02x})");
            }
        }
    }

    #[inline(always)]
    pub fn write_internal_byte(&mut self, address: u16, byte: u8) {
        match address {
            0x0000..=0x3FFF => self.cartridge.write_byte(address, byte), // ROM BANK 00
            0x4000..=0x7FFF => self.cartridge.write_byte(address, byte), // ROM BANK 01-NN
            0x8000..=0x9FFF => self.vram[address as usize - 0x8000] = byte, // VRAM
            0xA000..=0xBFFF => self.cartridge.write_byte(address, byte), // External RAM
            0xC000..=0xCFFF => self.wram0[address as usize - 0xC000] = byte, // WRAM 0
            0xD000..=0xDFFF => self.wram1[address as usize - 0xD000] = byte, // WRAM 1
            0xE000..=0xEFFF => self.wram0[address as usize - 0xE000] = byte, // ECHO -> WRAM 0
            0xF000..=0xFDFF => self.wram1[address as usize - 0xF000] = byte, // ECHO -> WRAM 1
            0xFE00..=0xFE9F => self.oam[address as usize - 0xFE00] = byte, // OAM
            0xFEA0..=0xFEFF => {}                                        // Not usable
            0xFF00..=0xFF7F => self.io_regs[address as usize - 0xFF00] = byte, // IO regs
            0xFF80..=0xFFFE => self.hram[address as usize - 0xFF80] = byte, // HRAM
            0xFFFF => self.interrupts = byte,                            // Interruptsmake
        }
    }

    pub fn read_word(&self, address: u16) -> u16 {
        (self.read_byte(address) as u16)  // LSB first
            | (self.read_byte(address + 1) as u16) << 8 // MSB second
    }

    pub fn write_word(&mut self, address: u16, word: u16) {
        self.write_byte(address, word as u8); // LSB first
        self.write_byte(address + 1, (word >> 8) as u8); // MSB second
    }
}

pub trait BusIO {
    fn read_byte(&self, address: u16) -> u8;
    fn write_byte(&mut self, address: u16, byte: u8);
    fn write_internal_byte(&mut self, address: u16, byte: u8);
    fn read_word(&self, address: u16) -> u16;
    fn write_word(&mut self, address: u16, word: u16);
}

pub trait InterruptBus: BusIO {
    define_flags_accessors!(interrupt_flag, 0xFF0F, Interrupt);
    define_flags_accessors!(interrupt_enable, 0xFFFF, Interrupt);
}
impl BusIO for MemorySystem {
    fn read_byte(&self, address: u16) -> u8 {
        self.read_byte(address)
    }

    fn write_byte(&mut self, address: u16, byte: u8) {
        self.write_byte(address, byte)
    }

    fn write_internal_byte(&mut self, address: u16, byte: u8) {
        self.write_internal_byte(address, byte)
    }

    fn read_word(&self, address: u16) -> u16 {
        self.read_word(address)
    }

    fn write_word(&mut self, address: u16, word: u16) {
        self.write_word(address, word)
    }
}

impl CpuBus for MemorySystem {}
impl PpuBus for MemorySystem {}
impl TimerBus for MemorySystem {}
impl InterruptBus for MemorySystem {}
impl JoypadBus for MemorySystem {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timer::{DMG_DIV_INITIAL_VALUE, Timer};

    #[test]
    fn test_read_write_byte() {
        let mut memory = MemorySystem::default();

        let test_cases = vec![
            (0x8000, 0xAB, "at vram start $0"),
            (0x9FFF, 0x42, "at vram end $9FFF"),
            (0xFFFF, 0x55, "at the highest address"),
        ];

        for (address, value, description) in test_cases {
            memory.write_byte(address, value);
            assert_eq!(
                memory.read_byte(address),
                value,
                "Read byte should return the written value {}",
                description
            );
        }
    }

    #[test]
    fn test_read_write_word() {
        let mut memory = MemorySystem::default();
        let test_cases = vec![
            (0x8000, 0xAB, "at vram start $0"),
            (0x9FFE, 0x42, "at vram end $9FFE"),
            (0xFFFE, 0x5555, "at the highest address"),
        ];
        for (address, value, description) in test_cases {
            memory.write_word(address, value);
            assert_eq!(
                memory.read_word(address),
                value,
                "Read word should return the written value {}",
                description
            );

            assert_eq!(
                memory.read_byte(address),
                value as u8,
                "LSB should be at the given address"
            );
            assert_eq!(
                memory.read_byte(address + 1),
                (value >> 8) as u8,
                "MSB should be at the given address"
            );
        }
    }

    #[test]
    fn test_dma_transfer() {
        let mut memory = MemorySystem::default();
        memory.write_byte(0xC000, 80); // y position
        memory.write_byte(0xC001, 88); // x position
        memory.write_byte(0xC002, 1); // tile index
        memory.write_byte(0xC003, 0); // attributes

        // DMA transfer
        memory.write_byte(0xFF46, 0xC0);

        assert_eq!(memory.read_oam(0), 80);
        assert_eq!(memory.read_oam(1), 88);
        assert_eq!(memory.read_oam(2), 1);
        assert_eq!(memory.read_oam(3), 0);
    }

    #[test]
    fn test_time_div_reset() {
        let mut timer = Timer::default();
        let mut bus = MemorySystem::default();
        timer.reset(&mut bus);

        // Accumulate a few cycles for DIV
        assert_eq!(bus.div(), DMG_DIV_INITIAL_VALUE);
        timer.step(&mut bus, 255);
        timer.step(&mut bus, 1);
        assert_eq!(bus.div(), DMG_DIV_INITIAL_VALUE + 1);

        // Simulate writing to a DIV element
        bus.write_byte(0xFF04, 0x12);

        // Check that DIV is reset to 0
        assert_eq!(bus.div(), 0);

        // Check that the internal counter is reset, ensuring that 256 complete cycles are required to increment DIV.
        timer.step(&mut bus, 255);
        assert_eq!(bus.div(), 0);
        timer.step(&mut bus, 1);
        assert_eq!(bus.div(), 1);
    }

    #[test]
    fn test_echo_ram() {
        // WRAM0    : C000..CFFF
        // WRAM1    : D000..DFFF
        // ECHORAM  : E000..FDFF
        //    => E000..EFFF => WRAM0 [0..FFF]
        //    => F000..FDFF => WRAM1 [0..DFF]

        let mut bus = MemorySystem::default();
        for i in 0..=(0xFFF + 0xDFF) {
            let byte = (1 + i) as u8;

            // write WRAM => ECHO
            bus.write_byte(0xC000 + i, byte);
            assert_eq!(bus.read_byte(0xE000 + i), byte);

            // Write ECHO => WRAM
            let byte = byte.wrapping_add_signed(1);
            bus.write_byte(0xE000 + i, byte);
            assert_eq!(bus.read_byte(0xC000 + i), byte);
        }
    }
}
