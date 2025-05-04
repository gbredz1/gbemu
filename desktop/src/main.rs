use crate::app::App;
use iced::{run, window, Font, Point, Settings, Size};

mod app;
pub(crate) mod style;
pub(crate) mod theme;
pub(crate) mod views;
pub(crate) mod widgets;

use clap::Parser;
use log::info;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[derive(Debug)]
struct Args {
    #[arg(short, long)]
    rom_path: String,
    #[arg(short = 'b', long, default_value = "false")]
    use_boot_rom: bool,
}

fn main() -> iced::Result {
    dotenv::dotenv().ok();
    env_logger::builder().format_timestamp_nanos().init();

    let args = Args::parse();
    info!("{:?}", args);

    iced::application(App::title, App::update, App::view)
        .antialiasing(false)
        .subscription(App::subscription)
        .window(window::Settings {
            size: Size::new(910.0, 830.0),
            ..window::Settings::default()
        })
        .position(window::Position::Specific(Point::new(1000.0, 30.0)))
        .settings(Settings {
            default_font: Font::with_name("Liberation Mono"),
            ..Settings::default()
        })
        .run_with(move || {
            let mut app = App::default();
            if args.use_boot_rom {
                app.machine.use_boot_rom().expect("Failed to load boot rom");
            }
            app.machine.reset();
            app.machine
                .load_cartridge(args.rom_path.as_str())
                .expect("Failed to load cartridge");

            (app, iced::Task::none())
        })
}
