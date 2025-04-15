use gameboy::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    env_logger::builder().format_timestamp_nanos().init();

    let mut system = Machine::default();
    system.bus.load_cartridge("roms/Tetris (World) (Rev A).gb")?;

    loop {
        system.cycle()?;
    }
}
