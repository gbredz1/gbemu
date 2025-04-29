use crossterm::event::{self, Event, KeyCode, KeyEvent};
use gbrust_core::{CpuFlags, Machine};
use ratatui::DefaultTerminal;
use ratatui::prelude::*;
use ratatui::widgets::canvas::{Canvas, Points};
use ratatui::widgets::{Block, Paragraph, Wrap};
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
    app.step_by_step = true;
    app.set_logs(logs);
    app.load("roms/tetris.gb")?;

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
    step_by_step: bool,
}

impl App {
    pub fn set_logs(&mut self, logs: Arc<Mutex<Vec<String>>>) {
        self.logs = Some(logs);
    }

    pub fn load(&mut self, path: &str) -> io::Result<()> {
        self.machine.load_cartridge(path)?;
        self.machine.reset();

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
        if self.step_by_step {
            self.machine.step().expect("Error while stepping machine");
        } else {
            self.machine.update(delta).expect("Error while updating the machine");
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        const GB_WIDTH: usize = 160;
        const GB_HEIGHT: usize = 144;

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
        let screen_block = Canvas::default()
            .block(Block::default().title("Screen"))
            .paint(|ctx| {
                let frame_buffer = self.machine.frame();
                let mut color_0 = vec![];
                let mut color_1 = vec![];
                let mut color_2 = vec![];
                let mut color_3 = vec![];

                for y in 0..GB_WIDTH {
                    for x in 0..GB_HEIGHT {
                        let idx = y * GB_HEIGHT + x;

                        let bytes = frame_buffer[idx];
                        match bytes {
                            1 => color_1.push((x as f64, y as f64)),
                            2 => color_2.push((x as f64, y as f64)),
                            3 => color_3.push((x as f64, y as f64)),
                            _ => color_0.push((x as f64, y as f64)),
                        }
                    }
                }

                ctx.draw(&Points {
                    coords: &color_0,
                    color: Color::Rgb(155, 188, 15),
                });
                ctx.draw(&Points {
                    coords: &color_1,
                    color: Color::Rgb(139, 172, 15),
                });
                ctx.draw(&Points {
                    coords: &color_2,
                    color: Color::Rgb(48, 98, 48),
                });
                ctx.draw(&Points {
                    coords: &color_3,
                    color: Color::Rgb(15, 56, 15),
                });
            })
            .x_bounds([0.0, GB_WIDTH as f64])
            .y_bounds([0.0, GB_HEIGHT as f64]);
        frame.render_widget(screen_block, screen_area);

        // Debug infos
        let debug_info = format!(
            "FPS: {:.1}\n\n\
            AF: {:04X}  Flags: [{}{}{}{}]\n\
            BC: {:04X}  LCDC:  {:02X}\n\
            DE: {:04X}  STAT:  {:02X}\n\
            HL: {:04X}  LY:  {:02X}\n\
            SP: {:04X}\n\
            PC: {:04X}\n\n",
            self.fps,
            self.machine.cpu().af(),
            if self.machine.cpu().flag(CpuFlags::Z) { "Z" } else { "_" },
            if self.machine.cpu().flag(CpuFlags::N) { "N" } else { "_" },
            if self.machine.cpu().flag(CpuFlags::H) { "H" } else { "_" },
            if self.machine.cpu().flag(CpuFlags::C) { "C" } else { "_" },
            self.machine.cpu().bc(),
            self.machine.bus.read_byte(0xFF40), // LCDC
            self.machine.cpu().de(),
            self.machine.bus.read_byte(0xFF41), // STAT
            self.machine.cpu().hl(),
            self.machine.bus.read_byte(0xFF44), // LY
            self.machine.cpu().sp(),
            self.machine.cpu().pc(),
        );

        let debug_widget = Paragraph::new(debug_info).block(Block::bordered().title("Debug"));

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
        if !self.step_by_step && !event::poll(Duration::from_nanos(0))? {
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
            KeyCode::Char('*') => self.machine.reset(),
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
