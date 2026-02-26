mod screen_view;

use crate::screen_view::{SCREEN_HEIGHT, SCREEN_WIDTH, ScreenView};
use clap::Parser;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyboardEnhancementFlags, PushKeyboardEnhancementFlags};
use crossterm::terminal::supports_keyboard_enhancement;
use crossterm::{event, execute};
use gbemu_core::{JoypadButton, Machine};
use log::{debug, error};
use ratatui::DefaultTerminal;
use ratatui::prelude::*;
use ratatui::symbols::Marker;
use ratatui::widgets::canvas::Canvas;
use std::io;
use std::thread::sleep;
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[derive(Debug)]
struct Args {
    rom_path: Option<String>,
    #[arg(short = 'b', long, default_value = "false")]
    use_boot_rom: bool,
}

fn main() -> io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::builder().format_timestamp_nanos().init();

    if !supports_keyboard_enhancement()? {
        error!("Keyboard enhancement isn't supported");
    }

    let args = Args::parse();
    debug!("{:?}", args);

    let mut result = Ok(());
    let mut app = App::default();
    if args.use_boot_rom {
        result = app.machine.use_boot_rom();
    }
    if let Some(rom_path) = &args.rom_path {
        result = app.load(rom_path.as_str());
    }

    if result.is_ok() {
        let mut terminal = ratatui::init();

        let mut stdout = io::stdout();
        execute!(stdout, PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::all()))?;

        result = app.run(&mut terminal);
    }

    ratatui::restore();

    result
}

#[derive(Default)]
struct App {
    machine: Machine,
    exit: bool,
}

const GB_FRAME_DURATION: Duration = Duration::from_nanos(16_742_706); // 1/59.7275 s
impl App {
    pub fn load(&mut self, path: &str) -> io::Result<()> {
        self.machine.load_cartridge(path)?;
        self.machine.reset();

        Ok(())
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut delta = Duration::from_nanos(0);

        while !self.exit {
            let frame_start = Instant::now();

            self.handle_events()?;
            self.update(&delta);
            terminal.draw(|frame| self.draw(frame))?;

            delta = frame_start.elapsed();

            if delta < GB_FRAME_DURATION {
                sleep(GB_FRAME_DURATION - delta);
            }
        }
        Ok(())
    }

    fn update(&mut self, _delta: &Duration) {
        self.machine.step_frame().unwrap_or_else(|e| {
            error!("{}", e);
            (0, false)
        });
    }

    fn draw(&self, frame: &mut Frame) {
        let screen_block = Canvas::default()
            .x_bounds([0., SCREEN_WIDTH as f64])
            .y_bounds([0., SCREEN_HEIGHT as f64])
            .marker(Marker::HalfBlock)
            .paint(|ctx| {
                ctx.draw(&ScreenView::from(self.machine.frame()));
            });
        frame.render_widget(screen_block, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if !event::poll(Duration::from_nanos(0))? {
            return Ok(());
        }

        if let Event::Key(key_event) = event::read()? {
            self.handle_key_event(key_event);
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match (key_event.code, !key_event.is_release()) {
            (KeyCode::Esc, _) => self.exit(),
            (KeyCode::Char('*'), _) => self.machine.reset(),
            (KeyCode::Up, pressed) => self.machine.button_changed(JoypadButton::Up, pressed),
            (KeyCode::Down, pressed) => self.machine.button_changed(JoypadButton::Down, pressed),
            (KeyCode::Left, pressed) => self.machine.button_changed(JoypadButton::Left, pressed),
            (KeyCode::Right, pressed) => self.machine.button_changed(JoypadButton::Right, pressed),
            (KeyCode::Char('d'), pressed) => self.machine.button_changed(JoypadButton::A, pressed),
            (KeyCode::Char('f'), pressed) => self.machine.button_changed(JoypadButton::B, pressed),
            (KeyCode::Char('c'), pressed) => self.machine.button_changed(JoypadButton::Select, pressed),
            (KeyCode::Char('v'), pressed) => self.machine.button_changed(JoypadButton::Start, pressed),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}
