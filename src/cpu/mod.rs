mod addressing_mode;
mod cpu_bus;

pub(crate) use crate::cpu::cpu_bus::CpuBus;
use log::debug;
mod decoder;
mod instruction;

#[cfg(test)]
mod decoder_test;
mod display;

use crate::cpu::decoder::LR35902Decoder;

pub struct CPU {
    decoder: LR35902Decoder,

    pub a: u8,
    pub f: u8, // Flags (Z, N, H, C)
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
    pub halted: bool, // L'état du CPU (utile pour l'instruction HALT)
}

impl Default for CPU {
    fn default() -> Self {
        CPU {
            decoder: Default::default(),
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0xFFFE,
            pc: 0x0100,
            halted: false,
        }
    }
}
impl CPU {
    pub fn cycle(&mut self, bus: &impl CpuBus) -> Result<(), String> {
        if self.halted {
            return Ok(());
        }

        let opcode = self.pc_read_byte(bus);
        debug!("opcode: ${:02x}", opcode);
        let instruction = self.decoder.decode(opcode);
        debug!("instruction: {:?}", instruction);
        if let Some(_) = instruction {
            Ok(())
        } else {
            Err(format!("Opcode 0x{:02x} unknow", opcode))
        }
    }

    pub fn pc_read_byte(&mut self, bus: &impl CpuBus) -> u8 {
        debug!("pc: ${:04x}:", self.pc);
        let byte = bus.read_byte(self.pc);

        self.pc = self.pc.wrapping_add(1);

        byte
    }
    pub fn pc_read_word(&mut self, bus: &impl CpuBus) -> u16 {
        let low = self.pc_read_byte(bus);
        let high = self.pc_read_byte(bus);

        (low as u16) << 8 | high as u16
    }
}
