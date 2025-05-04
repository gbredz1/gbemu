use crate::views::*;
use crate::widgets::screen::Screen;
use crate::widgets::{screen, title_panel};
use gbrust_core::Machine;
use iced::alignment::Vertical;
use iced::keyboard::key::Named;
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Subscription, Task, keyboard, time, window};
use std::time::{Duration, Instant};

pub(crate) struct App {
    pub machine: Machine,
    last_update: Option<Instant>,
    is_running: bool,
    breakpoint_at: String,

    view_memory_state: view_memory::State,
    screen: Screen,
    total_cycles: u64,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick(Instant),
    TogglePlayback,
    Step,
    Reset,
    BreakpointRemove,
    BreakpointSet(u16),
    BreakpointInputChanged(String),

    Screen(screen::Message),
    CloseWindow,

    MemoryView(view_memory::Message),
    StepFrame,

    OpenFile,
}

impl Default for App {
    fn default() -> Self {
        Self {
            machine: Machine::default(),
            last_update: None,
            is_running: false,
            breakpoint_at: "00e9".into(),
            view_memory_state: view_memory::State::default(),
            screen: Screen::default(),
            total_cycles: 0,
        }
    }
}

impl App {
    pub fn title(&self) -> String {
        String::from("My App")
    }
    pub fn subscription(&self) -> Subscription<Message> {
        let tick = match self.is_running {
            true => time::every(Duration::from_millis(30)).map(Message::Tick),
            false => Subscription::none(),
        };

        let keyboard = keyboard::on_key_press(|key, _modifiers| match key.as_ref() {
            keyboard::Key::Named(Named::F7) => Some(Message::Step),
            keyboard::Key::Character("r") => Some(Message::Reset),
            keyboard::Key::Named(Named::F10) => Some(Message::StepFrame),
            keyboard::Key::Named(Named::Space) => Some(Message::TogglePlayback),
            keyboard::Key::Named(Named::Escape) => Some(Message::CloseWindow),
            _ => None,
        });

        Subscription::batch(vec![tick, keyboard])
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CloseWindow => window::get_latest().and_then(window::close),
            Message::Tick(_now) => {
                let (cycles, break_flag) = self.machine.step_frame().expect("Failed to update the machine");
                self.total_cycles += cycles as u64;
                if break_flag {
                    self.is_running = false;
                }

                self.update(Message::Screen(screen::Message::UpdateFrameBuffer))
            }
            Message::Step => {
                self.is_running = false;

                self.total_cycles += self.machine.step().expect("Failed to step") as u64;

                Task::none()
            }
            Message::StepFrame => {
                self.is_running = false;
                self.total_cycles += self.machine.step_frame().unwrap().0 as u64;

                Task::none()
            }

            // Breakpoints
            Message::BreakpointRemove => {
                self.machine.breakpoint_manager_mut().clear();
                Task::none()
            }
            Message::BreakpointSet(addr) => {
                self.is_running = true;
                self.machine.breakpoint_manager_mut().add_breakpoint(addr);

                Task::none()
            }
            Message::BreakpointInputChanged(content) => {
                self.breakpoint_at = content;

                Task::none()
            }

            // Rom load
            Message::OpenFile => {
                let dialog = native_dialog::DialogBuilder::file()
                    .set_title("Open file")
                    .add_filter("ROM", ["gb"])
                    .open_single_file();

                let result = dialog.show();

                if let Ok(Some(path)) = result {
                    self.machine.reset();
                    self.machine.load_cartridge(path).expect("Failed to load rom");
                };

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
                self.total_cycles = 0;

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
        let screen = title_panel(
            "MEMORY",
            container(self.screen.view(self.machine.frame()).map(Message::Screen))
                .padding(4)
                .into(),
        )
        .center_x(170);

        let memory = title_panel(
            "MEMORY",
            view_memory::view(&self.view_memory_state, &self.machine).map(Message::MemoryView),
        )
        .center_x(550)
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
    let step_frame_button = button("Frame(F10)")
        .on_press(Message::StepFrame)
        .style(button::secondary);
    let total_cycles = text(format!("Cycles: {}", app.total_cycles));

    let breakpoint_empty = app.machine.breakpoint_manager().len() == 0;
    let breakpoint_u16 = || {
        if breakpoint_empty {
            u16::from_str_radix(app.breakpoint_at.as_str(), 16)
                .map(Message::BreakpointSet)
                .ok()
        } else {
            Some(Message::BreakpointRemove)
        }
    };

    let breakpoint_controls = row![
        text("Breakpoint at: $"),
        text_input("Breakpoint", &app.breakpoint_at)
            .width(60)
            .on_input(Message::BreakpointInputChanged)
            .on_submit_maybe(breakpoint_u16()),
        button(if breakpoint_empty { "Go" } else { "Del" })
            .on_press_maybe(breakpoint_u16())
            .style(button::secondary),
    ]
    .align_y(Vertical::Center);

    let load_rom = button("Load ROM").style(button::secondary).on_press(Message::OpenFile);

    row![
        run_button,
        step_button,
        step_frame_button,
        reset_button,
        breakpoint_controls,
        load_rom,
        total_cycles
    ]
    .spacing(8)
    .align_y(Vertical::Center)
    .into()
}
