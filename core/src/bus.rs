use crate::cpu::CpuBus;
use crate::ppu::PpuBus;
use bitflags::bitflags;
use std::default::Default;
use std::fs::File;
use std::io::Read;

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
pub(crate) use define_palette_accessors;

pub struct MemorySystem {
    memory: Vec<u8>,
}

impl MemorySystem {
    pub(crate) fn reset(&mut self) {
        // Clear VRAM
        self.memory[0x8000..=0x9fff].fill(0);
    }
}

impl Default for MemorySystem {
    fn default() -> Self {
        Self {
            memory: vec![0xFFu8; 0x1_0000],
        }
    }
}

impl MemorySystem {
    pub fn load_cartridge(&mut self, path: &str) -> Result<usize, std::io::Error> {
        let mut file = File::open(path)?;
        let mut rom = vec![];
        let size = file.read_to_end(&mut rom)?;
        self.memory[..size].copy_from_slice(&rom[..size]);

        Ok(size)
    }

    #[inline(always)]
    pub fn read_byte(&self, address: u16) -> u8 {
        unsafe { *self.memory.get_unchecked(address as usize) }
    }
    #[inline(always)]
    pub fn write_byte(&mut self, address: u16, byte: u8) {
        unsafe {
            *self.memory.get_unchecked_mut(address as usize) = byte;
        }
    }

    pub fn read_word(&self, address: u16) -> u16 {
        (self.memory[address as usize] as u16)  // LSB first
            | (self.memory[address as usize + 1] as u16) << 8 // MSB second
    }

    pub fn write_word(&mut self, address: u16, word: u16) {
        unsafe {
            *self.memory.get_unchecked_mut(address as usize) = word as u8; // LSB first
            *self.memory.get_unchecked_mut(address as usize + 1) = (word >> 8) as u8; // MSB second
        }
    }
}

pub trait BusIO {
    fn read_byte(&self, address: u16) -> u8;
    fn write_byte(&mut self, address: u16, byte: u8);
    fn read_word(&self, address: u16) -> u16;
    fn write_word(&mut self, address: u16, word: u16);
}

pub trait InterruptBus: BusIO {
    define_flags_accessors!(interrupt_flag, 0xFF0F, Interrupt);
    define_flags_accessors!(interrupt_enable, 0xFFFF, Interrupt);
}
impl BusIO for MemorySystem {
    fn read_byte(&self, addr: u16) -> u8 {
        self.read_byte(addr)
    }

    fn write_byte(&mut self, addr: u16, byte: u8) {
        self.write_byte(addr, byte);
    }

    fn read_word(&self, addr: u16) -> u16 {
        self.read_word(addr)
    }

    fn write_word(&mut self, addr: u16, word: u16) {
        self.write_word(addr, word);
    }
}

impl CpuBus for MemorySystem {}
impl PpuBus for MemorySystem {}
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
                "{}",
                format!("Read byte should return the written value {}", description)
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
                "{}",
                format!("Read word should return the written value {}", description)
            );

            assert_eq!(
                memory.read_byte(address),
                value as u8,
                "{}",
                "LSB should be at the given address"
            );
            assert_eq!(
                memory.read_byte(address + 1),
                (value >> 8) as u8,
                "{}",
                "MSB should be at the given address"
            );
        }
    }
}
