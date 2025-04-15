use crate::cpu::CpuBus;
use log::debug;
use std::default::Default;
use std::fs::File;
use std::io::Read;

pub struct Bus {
    memory: Vec<u8>,
}

impl Default for Bus {
    fn default() -> Self {
        Self {
            memory: vec![0xFFu8; 0x1_0000],
        }
    }
}

impl Bus {
    pub fn load_cartridge(&mut self, path: &str) -> Result<usize, std::io::Error> {
        let mut file = File::open(path)?;
        let mut rom = vec![];
        let size = file.read_to_end(&mut rom)?;
        for addr in 0..size {
            self.memory[addr] = rom[addr];
        }

        Ok(size)
    }
}

impl CpuBus for Bus {
    fn read_byte(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn write_byte(&mut self, addr: u16, byte: u8) {
        debug!("WRITE.B #{:04x}: {:02x}", addr, byte);
        self.memory[addr as usize] = byte;
    }

    fn read_word(&self, addr: u16) -> u16 {
        (self.memory[addr as usize] as u16) << 8 | self.memory[addr as usize + 1] as u16
    }

    fn write_word(&mut self, addr: u16, word: u16) {
        debug!("WRITE.W #{:04x}: {:04x}", addr, word);

        self.memory[addr as usize] = (word >> 8) as u8;
        self.memory[addr as usize + 1] = word as u8;
    }
}
