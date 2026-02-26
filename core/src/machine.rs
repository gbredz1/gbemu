use crate::bus::{InterruptBus, MemorySystem};
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::debug::breakpoint::BreakpointManager;
use crate::joypad;
use crate::joypad::Joypad;
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
    joypad: Joypad,
    start_addr: Option<u16>,
    breakpoint_manager: BreakpointManager,
}

impl Machine {
    pub fn use_boot_rom(&mut self) -> Result<(), std::io::Error> {
        self.start_addr = Some(0x0000);
        self.bus.load_boot_rom()
    }
    pub fn load_cartridge<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        info!("Loading cartridge: {:?}", path.as_ref());
        self.bus.load_cartridge(path)
    }

    pub fn frame(&self) -> &[u8] {
        &self.ppu.frame_buffer
    }
    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }
    pub fn bus(&self) -> &MemorySystem {
        &self.bus
    }
    pub fn cartridge(&self) -> &Cartridge {
        self.bus.cartridge()
    }

    pub fn breakpoint_manager(&self) -> &BreakpointManager {
        &self.breakpoint_manager
    }

    pub fn breakpoint_manager_mut(&mut self) -> &mut BreakpointManager {
        &mut self.breakpoint_manager
    }

    pub fn step_frame(&mut self) -> Result<(usize, bool), Box<dyn Error>> {
        const CYCLES_PER_FRAME: usize = 70224;

        let mut total_cycles: usize = 0;
        let mut breakpoint_hit = false;

        for _ in 0..CYCLES_PER_FRAME {
            total_cycles += self.step()? as usize;

            if self.breakpoint_manager.has_breakpoint(self.cpu.pc()) {
                breakpoint_hit = true;
                break;
            }
        }

        Ok((total_cycles, breakpoint_hit))
    }

    pub fn step(&mut self) -> Result<u8, Box<dyn Error>> {
        let cycles = self.cpu.step(&mut self.bus)?;
        self.ppu.update(&mut self.bus, cycles as u32);
        if !self.cpu.stop() {
            self.timer.step(&mut self.bus, cycles);
        }
        self.joypad.update(&mut self.bus);

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
        self.joypad.reset(&mut self.bus);

        self.bus.set_interrupt_enable_u8(0x00);
        self.bus.set_interrupt_flag_u8(0xE1);
    }

    pub fn button_pressed(&mut self, button: joypad::Button) {
        self.joypad.button_pressed(button);
    }

    pub fn button_released(&mut self, button: joypad::Button) {
        self.joypad.button_released(button);
    }

    pub fn button_changed(&mut self, button: joypad::Button, pressed: bool) {
        if pressed {
            self.button_pressed(button);
        } else {
            self.button_released(button);
        }
    }
}
