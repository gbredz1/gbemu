use iced::ContentFit::Fill;
use iced::application::Update;
use iced::event::Status;
use iced::mouse::Cursor;
use iced::widget::canvas::{Cache, Event, Geometry, Path, Program, event};
use iced::widget::{Canvas, canvas};
use iced::{Color, Element, Point, Size};
use iced::{Rectangle, Renderer, Theme};
use log::debug;

pub struct Screen {
    cache: Cache,
    frame_buffer: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum Message {
    UpdateFrameBuffer(Vec<u8>),
}

impl Screen {
    const SCREEN_WIDTH: usize = 160;
    const SCREEN_HEIGHT: usize = 144;

    pub fn new() -> Self {
        Self {
            cache: Cache::default(),
            frame_buffer: vec![0; Self::SCREEN_WIDTH * Self::SCREEN_HEIGHT],
        }
    }
    pub fn update(&mut self, message: Message) {
        match message {
            Message::UpdateFrameBuffer(frame_buffer) => {
                self.frame_buffer = frame_buffer;
                self.cache.clear();
            }
        }
    }
    pub fn view(&self) -> Element<Message> {
        Canvas::new(self).width(160).height(144).into()
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    fn color(val: u8) -> Color {
        match val {
            0 => Color::from_rgb8(155, 188, 15),
            1 => Color::from_rgb8(139, 172, 15),
            2 => Color::from_rgb8(48, 98, 48),
            _ => Color::from_rgb8(15, 56, 15),
        }
    }
}

impl Program<Message> for Screen {
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
            let background = Path::rectangle(
                Point::from([0f32, 0f32]),
                Size::new(Self::SCREEN_WIDTH as f32, Self::SCREEN_HEIGHT as f32),
            );
            frame.fill(&background, Color::from_rgb8(0, 0, 0));
            for x in 0..Self::SCREEN_WIDTH {
                for y in 0..Self::SCREEN_HEIGHT {
                    let point = Point::from([x as f32, y as f32]);
                    let index = x + (Self::SCREEN_WIDTH * y);
                    let color = Self::color(self.frame_buffer[index]);
                    let size = Size::new(1.0, 1.0);
                    frame.fill_rectangle(point, size, color)
                }
            }
        });
        vec![draw]
    }
}
