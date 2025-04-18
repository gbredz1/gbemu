use crate::bus::Bus;
use crate::cpu::Cpu;
use std::error::Error;

#[derive(Default)]
pub struct Machine {
    pub cpu: Cpu,
    pub bus: Bus,
}

impl Machine {
    pub fn cycle(&mut self) -> Result<bool, Box<dyn Error>> {
        self.cpu.cycle(&mut self.bus)?;

        Ok(true)
    }
}
