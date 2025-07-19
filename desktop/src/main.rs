use crate::app::{App, Message};
use iced::{Font, Point, Settings, Size, Theme, window};

mod app;
pub(crate) mod style;
pub(crate) mod theme;
pub(crate) mod views;
pub(crate) mod widgets;

use clap::Parser;
use font_kit::source::SystemSource;
use log::debug;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[derive(Debug)]
struct Args {
    rom_path: Option<String>,
    #[arg(short = 'b', long, default_value = "false")]
    use_boot_rom: bool,
    #[arg(long = "run", default_value = "false")]
    auto_run: bool,
}

fn main() -> iced::Result {
    dotenv::dotenv().ok();
    env_logger::builder().format_timestamp_nanos().init();

    let args = Args::parse();
    debug!("{:?}", args);

    let default_font = match SystemSource::new().select_family_by_name("Liberation Mono") {
        Ok(_) => Font::with_name("Liberation Mono"),
        Err(_) => Font::MONOSPACE,
    };

    iced::application(App::title, App::update, App::view)
        .antialiasing(false)
        .subscription(App::subscription)
        .theme(move |_| Theme::Dark) // force dark
        .window(window::Settings {
            size: Size::new(910.0, 830.0),
            ..window::Settings::default()
        })
        .position(window::Position::Specific(Point::new(1000.0, 30.0)))
        .settings(Settings {
            default_font,
            ..Settings::default()
        })
        .run_with(move || {
            let mut app = App::default();
            if args.use_boot_rom {
                app.machine.use_boot_rom().expect("Failed to load boot rom");
            }
            app.machine.reset();

            if let Some(rom_path) = &args.rom_path {
                app.machine
                    .load_cartridge(rom_path.as_str())
                    .expect("Failed to load cartridge");
            }

            let command = if args.auto_run {
                iced::Task::done(Message::TogglePlayback)
            } else {
                iced::Task::none()
            };

            (app, command)
        })
}
