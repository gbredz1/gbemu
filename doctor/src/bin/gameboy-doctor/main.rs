use clap::Parser;
use gbemu_core::{MemorySystem, Timer};
use log::debug;
use std::error::Error;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[derive(Debug)]
struct Args {
    rom_path: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    env_logger::builder().init();

    let args = Args::parse();
    debug!("{:?}", args);

    let mut cpu = gbemu_core::Cpu::default();
    let mut bus = MemorySystem::default();
    let mut timer = Timer::default();

    bus.load_cartridge(args.rom_path)?;
    cpu.reset();

    bus.write_byte(0xFF44, 0x90); // LY = 90

    let mut serial_buffer = String::new();

    loop {
        println!(
            "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}",
            cpu.a(),
            cpu.f(),
            cpu.b(),
            cpu.c(),
            cpu.d(),
            cpu.e(),
            cpu.h(),
            cpu.l(),
            cpu.sp(),
            cpu.pc(),
            bus.read_byte(cpu.pc()),
            bus.read_byte(cpu.pc().wrapping_add(1)),
            bus.read_byte(cpu.pc().wrapping_add(2)),
            bus.read_byte(cpu.pc().wrapping_add(3)),
        );

        let cycles = cpu.step(&mut bus)?;
        timer.step(&mut bus, cycles);

        if simple_serial(&mut bus, &mut serial_buffer) {
            break;
        }
    }

    Ok(())
}

fn simple_serial(bus: &mut MemorySystem, serial_buffer: &mut String) -> bool {
    let sc = bus.read_byte(0xFF00);
    if sc & 0b1000_0000 != 0 {
        let sb = bus.read_byte(0xFF01);
        bus.write_byte(0xFF01, 0xFF);

        match sb {
            0x0A => {
                debug!("[SERIAL] => {}", serial_buffer.trim());

                match serial_buffer.trim().to_lowercase().as_str() {
                    "passed" => return true,
                    s => {
                        if s.starts_with("failed") {
                            return true;
                        }
                    }
                }

                serial_buffer.clear();
            }
            0xFF => {}
            _ => {
                serial_buffer.push(sb as char);
            }
        }
    }
    false
}
