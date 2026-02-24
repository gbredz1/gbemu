use iced::mouse::Cursor;
use iced::widget::canvas;
use iced::widget::canvas::Geometry;
use iced::{Color, Element, Point, Size, Task};
use iced::{Rectangle, Renderer, Theme};

#[derive(Default)]
pub struct Screen {
    cache: canvas::Cache,
}

#[derive(Debug, Clone)]
pub enum Message {
    UpdateFrameBuffer,
}

impl Screen {
    pub const WIDTH: usize = 160;
    pub const HEIGHT: usize = 144;

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UpdateFrameBuffer => self.clear(),
        }

        Task::none()
    }
    pub fn view<'a>(&'a self, frame_buffer: &'a Vec<u8>) -> Element<'a, Message> {
        canvas(ScreenCanvas {
            cache: &self.cache,
            frame_buffer,
        })
        .width(Self::WIDTH as f32)
        .height(Self::HEIGHT as f32 + 1.0)
        .into()
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

struct ScreenCanvas<'a> {
    cache: &'a canvas::Cache,
    frame_buffer: &'a Vec<u8>,
}
impl<'a> canvas::Program<Message> for ScreenCanvas<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let draw = self.cache.draw(renderer, bounds.size(), |frame| {
            let background = canvas::Path::rectangle(
                Point::from([0f32, 0f32]),
                Size::new(Screen::WIDTH as f32, Screen::HEIGHT as f32),
            );
            frame.fill(&background, Color::from_rgb8(15, 56, 15));

            for x in 0..Screen::WIDTH {
                for y in 0..Screen::HEIGHT {
                    let point = Point::from([x as f32, y as f32]);
                    let index = x + (Screen::WIDTH * y);

                    let color = self.frame_buffer[index];
                    if color > 2 {
                        continue;
                    }
                    let color = match color {
                        0 => Color::from_rgb8(155, 188, 15),
                        1 => Color::from_rgb8(139, 172, 15),
                        2 => Color::from_rgb8(48, 98, 48),
                        _ => Color::from_rgb8(15, 56, 15), // background color
                    };
                    let size = Size::new(1.0, 1.0);
                    frame.fill_rectangle(point, size, color)
                }
            }
        });
        vec![draw]
    }
}
