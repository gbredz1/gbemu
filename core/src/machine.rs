use crate::bus::{InterruptBus, MemorySystem};
use crate::cpu::Cpu;
use crate::debug::breakpoint::BreakpointManager;
use crate::ppu::Ppu;
use crate::timer::Timer;
use log::info;
use std::error::Error;
use std::path::Path;

#[derive(Default)]
pub struct Machine {
    cpu: Cpu,
    bus: MemorySystem,
    ppu: Ppu,
    timer: Timer,
    start_addr: Option<u16>,
    breakpoint_manager: BreakpointManager,
}

impl Machine {
    pub fn use_boot_rom(&mut self) -> Result<(), std::io::Error> {
        self.start_addr = Some(0x0000);
        self.bus.load_boot_rom()
    }
    pub fn load_cartridge<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, std::io::Error> {
        info!("Loading cartridge: {:?}", path.as_ref());
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

    pub fn breakpoint_manager(&self) -> &BreakpointManager {
        &self.breakpoint_manager
    }

    pub fn breakpoint_manager_mut(&mut self) -> &mut BreakpointManager {
        &mut self.breakpoint_manager
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

        Ok((total_cycles as usize, break_flag))
    }

    pub fn step(&mut self) -> Result<u8, Box<dyn Error>> {
        let cycles = self.cpu.step(&mut self.bus)?;
        self.timer.step(&mut self.bus, cycles);
        self.ppu.update(&mut self.bus, cycles as u32);

        Ok(cycles)
    }

    pub fn reset(&mut self) {
        info!("Resetting");
        self.bus.reset();
        self.cpu.reset();
        if let Some(addr) = self.start_addr {
            self.cpu.set_pc(addr);
        }
        self.timer.reset(&mut self.bus);
        self.ppu.reset(&mut self.bus);

        self.bus.set_interrupt_enable_u8(0x00);
        self.bus.set_interrupt_flag_u8(0xE1);
    }
}
