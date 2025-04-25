use crate::theme::color;
use iced::Theme;
use iced::widget::text::Style;

pub fn reg8(_: &Theme) -> Style {
    Style {
        color: Some(color::orange()),
    }
}

pub fn reg16(_: &Theme) -> Style {
    Style {
        color: Some(color::blue()),
    }
}

pub fn reg_flag(_: &Theme) -> Style {
    Style {
        color: Some(color::green()),
    }
}
