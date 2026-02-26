use ratatui::style::Color;
use ratatui::widgets::canvas::{Painter, Shape};

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

pub struct ScreenView<'a> {
    image: &'a [u8],
}

impl<'a> From<&'a [u8]> for ScreenView<'a> {
    fn from(image: &'a [u8]) -> Self {
        Self { image }
    }
}

impl Shape for ScreenView<'_> {
    fn draw(&self, painter: &mut Painter) {
        self.image.iter().enumerate().for_each(|(index, &v)| {
            let x = index % SCREEN_WIDTH;
            let y = index / SCREEN_WIDTH;

            let Some((x, y)) = painter.get_point(x as f64, (SCREEN_HEIGHT - y) as f64) else {
                return;
            };

            let color = match &v {
                0 => Color::Rgb(155, 188, 15),
                1 => Color::Rgb(139, 172, 15),
                2 => Color::Rgb(48, 98, 48),
                _ => Color::Rgb(15, 56, 15), // background
            };

            painter.paint(x, y, color);
        });
    }
}
