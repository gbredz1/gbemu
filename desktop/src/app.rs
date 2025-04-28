use crate::views::*;
use crate::widgets::screen::Screen;
use crate::widgets::{screen, title_panel};
use gbrust_core::Machine;
use iced::alignment::Vertical;
use iced::keyboard::key::Named;
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{button, column, row, scrollable, text, text_input};
use iced::{Element, Subscription, Task, keyboard, time, window};
use log::debug;
use std::time::{Duration, Instant};

pub(crate) struct App {
    machine: Machine,
    last_update: Option<Instant>,
    is_running: bool,
    screen: Screen,
    breakpoint_at: String,

    view_memory_state: view_memory::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick(Instant),
    TogglePlayback,
    Step,
    Reset,
    StepToAddr(u16),
    StepToAddrInputChanged(String),
    Screen(screen::Message),
    CloseWindow,

    MemoryView(view_memory::Message),
}

impl Default for App {
    fn default() -> Self {
        let mut device = Machine::default();
        device
            .load_cartridge("roms/tetris.gb")
            .expect("Failed to load cartridge");
        device.reset();

        Self {
            machine: device,
            last_update: None,
            is_running: false,
            breakpoint_at: "021D".into(),
            screen: Screen::new(),
            view_memory_state: view_memory::State::default(),
        }
    }
}

impl App {
    pub fn title(&self) -> String {
        String::from("My App")
    }
    pub fn subscription(&self) -> Subscription<Message> {
        let tick = match self.is_running {
            true => time::every(Duration::from_millis(17)).map(Message::Tick),
            false => Subscription::none(),
        };
        let keyboard = keyboard::on_key_press(|key, _modifiers| match key.as_ref() {
            keyboard::Key::Named(Named::F7) => Some(Message::Step),
            keyboard::Key::Character("r") => Some(Message::Reset),
            keyboard::Key::Named(Named::Space) => Some(Message::TogglePlayback),
            keyboard::Key::Named(Named::Escape) => Some(Message::CloseWindow),
            _ => None,
        });

        Subscription::batch(vec![tick, keyboard])
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CloseWindow => window::get_latest().and_then(window::close),
            Message::Tick(now) => {
                let last_update = self.last_update.unwrap_or(now);
                let _delta = now - last_update;
                self.last_update = Some(now);

                // self.machine.update(&delta).expect("Failed to update machine");

                // if vblank occurred or simple 60Hz update
                let frame_buffer = self.machine.frame().clone();
                self.update(Message::Screen(screen::Message::UpdateFrameBuffer(frame_buffer)))
            }
            Message::Step => {
                self.is_running = false;

                self.machine.step().expect("Failed to step");

                // if vblank occured or simple 60Hz update
                // let frame_buffer = self.machine.frame().clone();
                // self.update(Message::Screen(screen::Message::UpdateFrameBuffer(frame_buffer)))

                Task::none()
            }
            Message::StepToAddr(addr) => {
                self.is_running = false;
                debug!("Step over to: {:04X}", addr);

                loop {
                    self.machine.step().expect("Failed to step");
                    if self.machine.cpu().pc() == addr {
                        break;
                    }
                }

                Task::none()
            }
            Message::StepToAddrInputChanged(content) => {
                self.breakpoint_at = content;

                Task::none()
            }

            Message::TogglePlayback => {
                self.is_running = !self.is_running;
                if !self.is_running {
                    self.last_update = None;
                }

                Task::none()
            }
            Message::Reset => {
                self.machine.reset();
                self.screen.clear();

                Task::none()
            }

            Message::Screen(msg) => {
                self.screen.update(msg);

                Task::none()
            }

            Message::MemoryView(msg) => self.view_memory_state.update(msg).map(Message::MemoryView),
        }
    }
    pub fn view(&self) -> Element<Message> {
        let controls = view_control(self.is_running, self);
        let cpu_state = title_panel("CPU", view_cpu::view(self.machine.cpu())).center_x(200);
        let io_registers = title_panel("IO REGISTERS", view_registers::view(&self.machine)).center_x(500);
        let screen = title_panel("SCREEN", self.screen.view().map(Message::Screen)).center_x(162);
        let memory = title_panel(
            "MEMORY",
            view_memory::view(&self.view_memory_state, &self.machine).map(Message::MemoryView),
        )
        .center_x(460)
        .height(370);

        let content = column![controls, row![cpu_state, io_registers, screen].spacing(10), memory]
            .spacing(10)
            .padding(10);
        Element::from(scrollable(content).direction(Direction::Both {
            vertical: Scrollbar::default(),
            horizontal: Scrollbar::default(),
        })) //.explain(Color::from_rgb8(252, 15, 192))
    }
}

fn view_control<'a>(is_running: bool, app: &App) -> Element<'a, Message> {
    let run_button = button(if is_running { "Pause" } else { "Play" })
        .on_press(Message::TogglePlayback)
        .style(button::primary);
    let step_button = button("Step(F7)").on_press(Message::Step).style(button::secondary);
    let reset_button = button("Reset(R)").on_press(Message::Reset).style(button::secondary);

    let breakpoint_u16 = || {
        u16::from_str_radix(app.breakpoint_at.as_str(), 16)
            .map(Message::StepToAddr)
            .ok()
    };

    let step_to = row![
        text("Breakpoint at: $"),
        text_input("Breakpoint", &app.breakpoint_at)
            .width(60)
            .on_input(Message::StepToAddrInputChanged)
            .on_submit_maybe(breakpoint_u16()),
        button("Go").on_press_maybe(breakpoint_u16()).style(button::secondary),
    ]
    .align_y(Vertical::Center);

    row![run_button, step_button, reset_button, step_to]
        .spacing(8)
        .align_y(Vertical::Center)
        .into()
}
