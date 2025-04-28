use crate::app::App;
use iced::{Font, Point, Settings, Size, window};

mod app;
pub(crate) mod style;
pub(crate) mod theme;
pub(crate) mod views;
pub(crate) mod widgets;

fn main() -> iced::Result {
    dotenv::dotenv().ok();
    env_logger::builder().format_timestamp_nanos().init();

    iced::application(App::title, App::update, App::view)
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
        .run()
}
