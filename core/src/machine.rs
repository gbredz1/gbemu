use crate::bus::MemorySystem;
use crate::cpu::Cpu;
use crate::ppu::Ppu;
use std::error::Error;
use std::time::Duration;

#[derive(Default)]
pub struct Machine {
    cpu: Cpu,
    pub bus: MemorySystem,
    ppu: Ppu,

    // timing
    accumulator: i64,
}

impl Machine {
    const CPU_STEP_NS: i64 = 238; // ~4194304 Hz

    pub fn load_cartridge(&mut self, path: &str) -> Result<usize, std::io::Error> {
        self.bus.load_cartridge(path)
    }

    pub fn frame(&self) -> &Vec<u8> {
        &self.ppu.frame_buffer
    }
    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn update(&mut self, delta: &Duration) -> Result<(), Box<dyn Error>> {
        self.accumulator += delta.as_nanos() as i64;

        while self.accumulator >= Self::CPU_STEP_NS {
            let cycles = self.step()?;
            self.accumulator -= Self::CPU_STEP_NS * cycles as i64;
        }

        Ok(())
    }

    pub fn step(&mut self) -> Result<usize, Box<dyn Error>> {
        let cycles = self.cpu.step(&mut self.bus)?;
        self.ppu.update(&mut self.bus, cycles as u32);

        Ok(cycles)
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.ppu.reset(&mut self.bus);
    }
}
