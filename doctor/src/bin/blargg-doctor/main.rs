use clap::Parser;
use gbemu_core::serial::serial_bus::{SerialBus, SC};
use gbemu_core::Machine;
use log::debug;
use std::error::Error;
use std::process::exit;

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

    let mut machine = Machine::default();
    machine.load_cartridge(&args.rom_path)?;
    machine.reset();

    let mut buffer = String::new();
    loop {
        machine.step()?;

        if let Some(value) = serial(&mut machine, &mut buffer) {
            exit(value);
        }
    }
}

fn serial(machine: &mut Machine, buffer: &mut String) -> Option<i32> {
    let bus = machine.bus_mut() as &mut dyn SerialBus;
    if bus.sc().contains(SC::Enable) {
        let data = bus.sb(); // read
        bus.set_sb(0xFF); // clear

        match data {
            0x0A => {
                println!("[SERIAL]> {}", buffer.trim());
                let result = buffer.to_string().to_lowercase();
                buffer.clear();

                if result.starts_with("passed") {
                    return Some(0);
                }
                if result.starts_with("failed") {
                    return Some(1);
                }
            }
            0xFF => {}
            _ => {
                buffer.push(data as char);
            }
        }
    }
    None
}
