use iced::{Font, Settings};
use crate::app::App;

mod app;
mod style;
mod views;
mod widgets;
mod theme;

fn main() -> iced::Result {
    dotenv::dotenv().ok();
    env_logger::builder().format_timestamp_nanos().init();

    iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .settings(Settings {
            default_font: Font::with_name("Liberation Mono"),
            ..Settings::default()
        })
        .run()
}
