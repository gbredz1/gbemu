mod addressing_mode;
mod cpu_bus;

use crate::cpu::addressing_mode::CC;
pub(crate) use crate::cpu::cpu_bus::CpuBus;
use bitflags::bitflags;
use log::debug;

mod decoder;
mod instruction;

#[cfg(test)]
mod decoder_test;
mod display;
mod register;

use crate::cpu_decode;

bitflags! {
    pub struct Flags: u8 {
        const Z = 0b1000_0000;
        const N = 0b0100_0000;
        const H = 0b0010_0000;
        const C = 0b0001_0000;
    }
}

pub struct CPU {
    pub a: u8,
    pub f: Flags, // Flags (Z, N, H, C)
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
    pub halted: bool,
}

impl Default for CPU {
    fn default() -> Self {
        CPU {
            a: 0,
            f: Flags::empty(),
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
    pub fn cycle(&mut self, bus: &mut impl CpuBus) -> Result<(), String> {
        if self.halted {
            return Ok(());
        }

        let opcode = self.pc_read_byte(bus);
        debug!("opcode: ${:02x}", opcode);
        let instruction = cpu_decode!(opcode);
        if let Some(instruction) = instruction {
            let mut data = vec![];
            for _ in 1..instruction.size {
                data.push(self.pc_read_byte(bus));
            }

            debug!(
                "exec: [{}]{}",
                instruction.operation,
                if !data.is_empty() {
                    format!(
                        " | data [{}]",
                        data.iter()
                            .map(|b| format!("0x{:02X}", b))
                            .collect::<Vec<String>>()
                            .join(", ")
                    )
                } else {
                    String::new()
                }
            );

            instruction.execute(self, bus, data);
            Ok(())
        } else {
            Err(format!("Opcode 0x{:02x} unknown", opcode))
        }
    }

    fn pc_read_byte(&mut self, bus: &impl CpuBus) -> u8 {
        let byte = bus.read_byte(self.pc);
        debug!("pc: ${:04x}: {:02x}", self.pc, byte);
        self.pc = self.pc.wrapping_add(1);

        byte
    }

    pub fn get_flag(&self, flag: Flags) -> bool {
        self.f.contains(flag)
    }
    pub fn set_flag(&mut self, flag: Flags) {
        self.f.insert(flag)
    }
    pub fn clear_flag(&mut self, flag: Flags) {
        self.f.remove(flag)
    }
    pub fn set_flag_if(&mut self, flag: Flags, condition: bool) {
        if condition {
            self.set_flag(flag);
        } else {
            self.clear_flag(flag);
        }
    }
    pub fn check_condition(&self, condition: CC) -> bool {
        match condition {
            CC::NZ => !self.get_flag(Flags::Z),
            CC::Z => self.get_flag(Flags::Z),
            CC::NC => !self.get_flag(Flags::C),
            CC::C => self.get_flag(Flags::C),
        }
    }
}
