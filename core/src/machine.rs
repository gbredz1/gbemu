use crate::bus::MemorySystem;
use crate::cpu::Cpu;
use crate::ppu::Ppu;
use std::error::Error;
use std::time::Duration;

#[derive(Default)]
pub struct Machine {
    pub cpu: Cpu,
    bus: MemorySystem,
    ppu: Ppu,

    // timing
    accumulator: i64,
}

impl Machine {
    const CPU_STEP_NS: i64 = 238_000_000; // ~4194304 Hz

    pub fn load_cartridge(&mut self, p0: &str) -> Result<usize, std::io::Error> {
        self.bus.load_cartridge(p0)
    }

    pub fn update(&mut self, delta: &Duration) -> Result<(), Box<dyn Error>> {
        self.accumulator += delta.as_nanos() as i64;

        while self.accumulator >= Self::CPU_STEP_NS {
            let cycles = self.cpu.tick(&mut self.bus)?;
            self.ppu.update(&mut self.bus, cycles as u32);

            self.accumulator -= Self::CPU_STEP_NS * cycles as i64;
        }

        Ok(())
    }
}
