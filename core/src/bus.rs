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
use crate::timer::timer_bus::TimerBus;
pub(crate) use define_palette_accessors;

pub struct MemorySystem {
    memory: Vec<u8>,
    boot_rom: [u8; 0x100],
    boot_rom_enabled: bool,
    boot_rom_loaded: bool,
}

impl MemorySystem {
    pub(crate) fn reset(&mut self) {
        // Clear VRAM
        self.memory[0x8000..=0x9fff].fill(0);
        self.boot_rom_enabled = self.boot_rom_loaded;
    }
}

impl Default for MemorySystem {
    fn default() -> Self {
        Self {
            memory: vec![0xFFu8; 0x1_0000],
            boot_rom_enabled: false,
            boot_rom_loaded: false,
            boot_rom: [0; 0x100],
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

    pub fn load_cartridge<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, std::io::Error> {
        let mut file = File::open(path)?;
        let mut rom = vec![];
        let size = file.read_to_end(&mut rom)?;
        self.memory[..size].copy_from_slice(&rom[..size]);

        Ok(size)
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        if self.boot_rom_enabled && address < 0x100 {
            unsafe { *self.boot_rom.get_unchecked(address as usize) }
        } else {
            unsafe { *self.memory.get_unchecked(address as usize) }
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
        unsafe {
            *self.memory.get_unchecked_mut(address as usize) = byte;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_write_byte() {
        let mut memory = MemorySystem::default();

        let test_cases = vec![
            (0x1234, 0xAB, "at a specific address"),
            (0x0000, 0x42, "at address 0"),
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
            (0x1234, 0xABCD, "at a specific address"),
            (0x0000, 0x4242, "at address 0"),
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
}
