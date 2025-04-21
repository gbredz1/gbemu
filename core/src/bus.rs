use crate::cpu::CpuBus;
use crate::ppu::PpuBus;
use bitflags::bitflags;
use log::trace;
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

pub struct MemorySystem {
    memory: Vec<u8>,
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
        trace!("WRITE.B #{:04x}: {:02x}", address, byte);
        unsafe {
            *self.memory.get_unchecked_mut(address as usize) = byte;
        }
    }
    pub fn read_word(&self, address: u16) -> u16 {
        (self.memory[address as usize] as u16) << 8 | self.memory[address as usize + 1] as u16
    }
    pub fn write_word(&mut self, address: u16, word: u16) {
        trace!("WRITE.W #{:04x}: {:04x}", address, word);
        unsafe {
            *self.memory.get_unchecked_mut(address as usize) = (word >> 8) as u8;
            *self.memory.get_unchecked_mut(address as usize + 1) = word as u8;
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
    fn interrupt_flag(&self) -> Interrupt {
        Interrupt::from_bits_truncate(self.read_byte(0xFF0F))
    }
    fn set_interrupt_flag(&mut self, value: Interrupt) {
        self.write_byte(0xFF0F, value.bits());
    }
    fn update_interrupt_flag(&mut self, flag: Interrupt, enabled: bool) {
        let current = self.interrupt_flag();
        if enabled {
            self.set_interrupt_flag(current | flag);
        } else {
            self.set_interrupt_flag(current & !flag);
        }
    }
    fn request_interrupt(&mut self, interrupt: Interrupt) {
        self.update_interrupt_flag(interrupt, true);
    }
    fn interrupt_enable(&self) -> Interrupt {
        Interrupt::from_bits_truncate(self.read_byte(0xFFFF))
    }
    fn set_interrupt_enable(&mut self, value: Interrupt) {
        self.write_byte(0xFFFF, value.bits());
    }
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
