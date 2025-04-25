use iced::Color;

pub struct ThemeColor {
    pub red: Color,
    pub green: Color,
    pub blue: Color,
    pub yellow: Color,
    pub orange: Color,
    pub purple: Color,
    pub pink: Color,
}

impl ThemeColor {
    pub const DEFAULT: Self = Self {
        red: Color::from_rgb(255.0 / 255.0, 99.0 / 255.0, 71.0 / 255.0),
        green: Color::from_rgb(90.0 / 255.0, 206.0 / 255.0, 167.0 / 255.0),
        blue: Color::from_rgb(30.0 / 255.0, 144.0 / 255.0, 250.0 / 255.0),
        orange: Color::from_rgb(250.0 / 255.0, 150.0 / 255.0, 20.0 / 255.0),
        yellow: Color::from_rgb(255.0 / 255.0, 215.0 / 255.0, 0.0 / 255.0),
        purple: Color::from_rgb(153.0 / 255.0, 50.0 / 255.0, 204.0 / 255.0),
        pink: Color::from_rgb(250.0 / 255.0, 20.0 / 255.0, 147.0 / 255.0),
    };
}

pub(crate) mod color {
    use crate::theme::ThemeColor;
    use iced::Color;

    pub fn orange() -> Color {
        ThemeColor::DEFAULT.orange
    }
    pub fn yellow() -> Color {
        ThemeColor::DEFAULT.yellow
    }

    pub fn blue() -> Color {
        ThemeColor::DEFAULT.blue
    }

    pub fn green() -> Color {
        ThemeColor::DEFAULT.green
    }
    pub fn red() -> Color {
        ThemeColor::DEFAULT.red
    }
    pub fn purple() -> Color {
        ThemeColor::DEFAULT.purple
    }
    pub fn pink() -> Color {
        ThemeColor::DEFAULT.pink
    }
}
