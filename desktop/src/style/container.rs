use iced::widget::container::Style;
use iced::{Color, Theme, border};

pub fn panel_title(theme: &Theme) -> Style {
    Style::default()
        .background(theme.palette().primary)
        .border(border::color(Color::WHITE).width(1))
}
pub fn panel_content(_: &Theme) -> Style {
    Style::default()
        .background(Color::from_rgba(0.1, 0.1, 0.1, 0.5))
        .border(border::color(Color::from_rgba(0.8, 0.8, 0.8, 1.0)).width(1))
}
