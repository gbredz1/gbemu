use crate::cpu::CpuBus;
use crate::ppu::PpuBus;
use log::trace;
use std::default::Default;
use std::fs::File;
use std::io::Read;

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
    fn read_byte(&self, address: u16) -> u8 {
        unsafe { *self.memory.get_unchecked(address as usize) }
    }
    #[inline(always)]
    fn write_byte(&mut self, address: u16, byte: u8) {
        trace!("WRITE.B #{:04x}: {:02x}", address, byte);
        unsafe {
            *self.memory.get_unchecked_mut(address as usize) = byte;
        }
    }
    fn read_word(&self, address: u16) -> u16 {
        (self.memory[address as usize] as u16) << 8 | self.memory[address as usize + 1] as u16
    }
    fn write_word(&mut self, address: u16, word: u16) {
        trace!("WRITE.W #{:04x}: {:04x}", address, word);
        unsafe {
            *self.memory.get_unchecked_mut(address as usize) = (word >> 8) as u8;
            *self.memory.get_unchecked_mut(address as usize + 1) = word as u8;
        }
    }
}

impl CpuBus for MemorySystem {
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

impl PpuBus for MemorySystem {
    fn read_byte(&self, address: u16) -> u8 {
        self.read_byte(address)
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        self.write_byte(address, value);
    }
}
