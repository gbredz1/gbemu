use iced::theme::Palette;

#[derive(Clone, Default, Debug, PartialEq)]
pub enum Theme {
    Light,
    #[default]
    Dark,
}

impl Theme {
    pub fn palette(&self) -> Palette {
        match self {
            Theme::Light => Palette::LIGHT,
            Theme::Dark => Palette::DARK,
        }
    }
}
