use crossterm::event::{self, Event, KeyCode, KeyEvent};
use gbrust_core::Machine;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::DefaultTerminal;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};

fn main() -> io::Result<()> {
    dotenv::dotenv().ok();
    let logs = Arc::new(Mutex::new(Vec::new()));
    let target = AppLogger {
        logs: Arc::clone(&logs),
    };
    env_logger::builder()
        .target(env_logger::Target::Pipe(Box::new(target)))
        .init();

    let mut app = App::default();
    app.set_logs(logs);
    app.load("roms/Tetris (World) (Rev A).gb")?;

    let mut terminal = ratatui::init();
    let app_result = app.run(&mut terminal);
    ratatui::restore();

    app_result
}

#[derive(Default)]
struct App {
    machine: Machine,
    exit: bool,
    logs: Option<Arc<Mutex<Vec<String>>>>,
    fps: f64,
}

impl App {
    pub fn set_logs(&mut self, logs: Arc<Mutex<Vec<String>>>) {
        self.logs = Some(logs);
    }

    pub fn load(&mut self, path: &str) -> io::Result<()> {
        self.machine.load_cartridge(path)?;

        Ok(())
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut delta = Duration::from_nanos(0);
        let target_frame_time = Duration::from_secs_f64(1.0 / 60.0);

        while !self.exit {
            let frame_start = Instant::now();

            self.handle_events()?;
            self.update(&delta)?;
            terminal.draw(|frame| self.draw(frame))?;

            delta = frame_start.elapsed();

            // sleep for loop a 60fps
            if delta < target_frame_time {
                sleep(target_frame_time - delta);
            }

            self.fps = 1.0 / frame_start.elapsed().as_secs_f64();
        }
        Ok(())
    }

    fn update(&mut self, delta: &Duration) -> io::Result<()> {
        self.machine.update(delta).expect("Error while updating machine");
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Percentage(80), // screen § debug
            Constraint::Percentage(20), // logs
        ]);
        let [content_area, logs_area] = vertical.areas(frame.area());
        let [screen_area, debug_area] = Layout::horizontal([
            Constraint::Percentage(50), // screen
            Constraint::Percentage(50), // debug infos
        ])
        .areas(content_area);

        // Screen
        let screen_block = Block::default().title("Screen");
        frame.render_widget(screen_block, screen_area);

        // Debug infos
        let debug_info = format!(
            "FPS: {:.1}\n\nPC: 0x{:04X}\nRegisters: A: {:02X}, F: {:02X}\nSP: 0x{:04X}",
            self.fps,
            self.machine.cpu.pc(),
            self.machine.cpu.a(),
            self.machine.cpu.f(),
            self.machine.cpu.sp()
        );

        let debug_widget = Paragraph::new(debug_info)
            .block(Block::bordered().title("Debug"))
            .wrap(Wrap { trim: true });

        frame.render_widget(debug_widget, debug_area);

        // Logs
        if let Some(logs) = &self.logs {
            let mut logs_vec = logs.lock().unwrap();
            let logs_text = logs_vec.join("");
            logs_vec.clear();

            let logs_widget = Paragraph::new(logs_text)
                .block(Block::bordered().title("Logs"))
                .wrap(Wrap { trim: true });

            frame.render_widget(logs_widget, logs_area);
        }
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
        match key_event.code {
            KeyCode::Esc => self.exit(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

pub struct AppLogger {
    logs: Arc<Mutex<Vec<String>>>,
}

impl io::Write for AppLogger {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Ok(log_str) = String::from_utf8(buf.to_vec()) {
            if let Ok(mut logs) = self.logs.lock() {
                logs.push(log_str);
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
