use crate::bus::{InterruptBus, MemorySystem};
use crate::cpu::Cpu;
use crate::debug::breakpoint::BreakpointManager;
use crate::ppu::Ppu;
use std::error::Error;

#[derive(Default)]
pub struct Machine {
    cpu: Cpu,
    bus: MemorySystem,
    ppu: Ppu,

    start_addr: Option<u16>,
    breakpoint_manager: BreakpointManager,
}

impl Machine {
    pub fn load_cartridge(&mut self, path: &str) -> Result<usize, std::io::Error> {
        self.bus.load_cartridge(path)
    }

    pub fn frame(&self) -> &Vec<u8> {
        &self.ppu.frame_buffer
    }
    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }
    pub fn bus(&self) -> &MemorySystem {
        &self.bus
    }

    pub fn set_start_addr(&mut self, addr: u16) {
        self.start_addr = Some(addr);
    }
    pub fn start_addr(&self) -> Option<u16> {
        self.start_addr
    }
    pub fn set_breakpoint(&mut self, addr: u16) {
        self.breakpoint_manager.clear();
        self.breakpoint_manager.add_breakpoint(addr);
    }

    pub fn step_frame(&mut self) -> Result<(usize, bool), Box<dyn Error>> {
        let mut total_cycles = 0;
        let mut break_flag = false;

        for _ in 0..70224 {
            total_cycles += self.step()?;

            if self.breakpoint_manager.has_breakpoint(self.cpu.pc()) {
                break_flag = true;
                break;
            }
        }

        Ok((total_cycles, break_flag))
    }

    pub fn step(&mut self) -> Result<usize, Box<dyn Error>> {
        let cycles = self.cpu.step(&mut self.bus)?;
        self.ppu.update(&mut self.bus, cycles as u32);

        Ok(cycles)
    }

    pub fn reset(&mut self) {
        self.bus.reset();
        self.cpu.reset();
        if let Some(addr) = self.start_addr {
            self.cpu.set_pc(addr);
        }
        self.ppu.reset(&mut self.bus);

        self.bus.set_interrupt_enable_u8(0x00);
        self.bus.set_interrupt_flag_u8(0xE1);
    }
}
