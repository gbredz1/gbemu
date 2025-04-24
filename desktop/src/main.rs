use crate::widgets::GameboyScreen;
use gbrust_core::Machine;
use iced::time::{self};
use iced::widget::{Column, Text};
use iced::{Element, Subscription};
use std::time::{Duration, Instant};

fn main() -> iced::Result {
    dotenv::dotenv().ok();
    env_logger::builder().format_timestamp_nanos().init();

    iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
    ScreenUpdated,
}

struct App {
    machine: Machine,
    last_update: Instant,

    gameboy_screen: GameboyScreen,
}

impl Default for App {
    fn default() -> Self {
        let mut machine = Machine::default();
        machine
            .load_cartridge("roms/tetris.gb")
            .expect("Failed to load cartridge");

        Self {
            machine,
            last_update: Instant::now(),
            gameboy_screen: GameboyScreen::new(),
        }
    }
}

impl App {
    fn title(&self) -> String {
        String::from("My App")
    }
    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(17)).map(|_| Message::Tick(Instant::now()))
    }
    fn update(&mut self, message: Message) {
        match message {
            Message::Tick(now) => {
                let delta = now - self.last_update;
                self.last_update = now;

                self.machine.update(&delta).expect("Failed to update machine");

                // if vblank occured or simple 60Hz update
                let frame_buffer = self.machine.frame().clone();
                self.gameboy_screen.update_frame_buffer(frame_buffer);
            }
            _ => {}
        }
    }
    fn view(&self) -> Element<Message> {
        Column::new()
            .push(Text::new("Hello World!"))
            .push(self.gameboy_screen.view().map(|_| Message::ScreenUpdated))
            .into()
    }
}

// ----------- //
pub mod widgets {
    use iced::mouse::Cursor;
    use iced::widget::canvas::{Cache, Geometry, Path};
    use iced::widget::{Canvas, canvas};
    use iced::{Color, Element, Point, Size};
    use iced::{Rectangle, Renderer, Theme};

    #[derive(Clone)]
    pub enum Message {
        FrameReady,
    }

    pub struct GameboyScreen {
        cache: Cache,
        frame_buffer: Vec<u8>,
    }

    impl GameboyScreen {
        const SCREEN_WIDTH: usize = 160;
        const SCREEN_HEIGHT: usize = 144;

        pub fn new() -> Self {
            Self {
                cache: Cache::default(),
                frame_buffer: vec![0; Self::SCREEN_WIDTH * Self::SCREEN_HEIGHT],
            }
        }
        pub fn update_frame_buffer(&mut self, buffer: Vec<u8>) {
            self.frame_buffer = buffer;
            self.cache.clear();
        }

        pub fn update(&mut self, message: Message) {}

        pub fn view(&self) -> Element<Message> {
            Canvas::new(self).width(160).height(144).into()
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

    impl canvas::Program<Message> for GameboyScreen {
        type State = ();

        fn draw(
            &self,
            state: &Self::State,
            renderer: &Renderer,
            theme: &Theme,
            bounds: Rectangle,
            cursor: Cursor,
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
}
