use crate::app::Message;
use crate::style::container::{panel_content, panel_title};
use iced::alignment::Horizontal;
use iced::widget::{Container, container, text};
use iced::{Element, Fill};

pub(crate) mod screen;

pub(crate) fn title_panel<'a>(name: &'a str, content: Element<'a, Message>) -> Container<'a, Message> {
    container(
        iced::widget::column![container(text(name).center().width(Fill)).style(panel_title), content,]
            .align_x(Horizontal::Center),
    )
    .style(panel_content)
}
