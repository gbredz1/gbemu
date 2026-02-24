use crate::views::*;
use crate::widgets::screen::Screen;
use crate::widgets::{screen, title_panel};
use gbemu_core::{JoypadButton, Machine};
use iced::alignment::{Horizontal, Vertical};
use iced::keyboard::key::Named;
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Subscription, Task, keyboard, time, window};
use iced_core::keyboard::{Event, Key};
use log::error;
use std::time::{Duration, Instant};

// Application constants
const DEFAULT_BREAKPOINT: &str = "00e9";
const UPDATE_INTERVAL_MS: u64 = 30;
const BUTTON_SPACING: f32 = 8.0;
const COLUMN_SPACING: f32 = 10.0;
const CONTENT_PADDING: f32 = 10.0;

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
    // Execution control
    Tick(Instant),
    TogglePlayback,
    Step,
    StepFrame,
    Reset,

    // User interface
    CloseWindow,
    OpenFile,

    // Breakpoint management
    BreakpointRemove,
    BreakpointSet(u16),
    BreakpointInputChanged(String),

    // Visual components
    ScreenView(screen::Message),
    MemoryView(view_memory::Message),

    // Machine inputs
    ButtonsPressed(JoypadButton),
    ButtonsReleased(JoypadButton),
}

impl Default for App {
    fn default() -> Self {
        Self {
            machine: Machine::default(),
            last_update: None,
            is_running: false,
            breakpoint_at: DEFAULT_BREAKPOINT.into(),
            view_memory_state: view_memory::State::default(),
            screen: Screen::default(),
            total_cycles: 0,
        }
    }
}

impl App {
    pub fn title(&self) -> String {
        String::from("Iced GB")
    }
    pub fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![];
        if self.is_running {
            subscriptions.push(time::every(Duration::from_millis(UPDATE_INTERVAL_MS)).map(Message::Tick));
        };

        subscriptions.push(keyboard::listen().filter_map(|event| {
            if let Event::KeyPressed {
                key,
                modifiers: _modifiers,
                ..
            } = event
            {
                match key.as_ref() {
                    Key::Named(Named::F7) => Some(Message::Step),
                    Key::Character("r") => Some(Message::Reset),
                    Key::Named(Named::F10) => Some(Message::StepFrame),
                    Key::Named(Named::Space) => Some(Message::TogglePlayback),
                    Key::Named(Named::Escape) => Some(Message::CloseWindow),
                    Key::Character("l") => Some(Message::OpenFile),

                    Key::Named(Named::ArrowUp) => Some(Message::ButtonsPressed(JoypadButton::Up)),
                    Key::Named(Named::ArrowDown) => Some(Message::ButtonsPressed(JoypadButton::Down)),
                    Key::Named(Named::ArrowLeft) => Some(Message::ButtonsPressed(JoypadButton::Left)),
                    Key::Named(Named::ArrowRight) => Some(Message::ButtonsPressed(JoypadButton::Right)),
                    Key::Character("d") => Some(Message::ButtonsPressed(JoypadButton::A)),
                    Key::Character("f") => Some(Message::ButtonsPressed(JoypadButton::B)),
                    Key::Character("c") => Some(Message::ButtonsPressed(JoypadButton::Start)),
                    Key::Character("v") => Some(Message::ButtonsPressed(JoypadButton::Select)),

                    _ => None,
                }
            } else if let Event::KeyReleased {
                key,
                modifiers: _modifiers,
                ..
            } = event
            {
                match key.as_ref() {
                    Key::Named(Named::ArrowUp) => Some(Message::ButtonsReleased(JoypadButton::Up)),
                    Key::Named(Named::ArrowDown) => Some(Message::ButtonsReleased(JoypadButton::Down)),
                    Key::Named(Named::ArrowLeft) => Some(Message::ButtonsReleased(JoypadButton::Left)),
                    Key::Named(Named::ArrowRight) => Some(Message::ButtonsReleased(JoypadButton::Right)),
                    Key::Character("d") => Some(Message::ButtonsReleased(JoypadButton::A)),
                    Key::Character("f") => Some(Message::ButtonsReleased(JoypadButton::B)),
                    Key::Character("c") => Some(Message::ButtonsReleased(JoypadButton::Start)),
                    Key::Character("v") => Some(Message::ButtonsReleased(JoypadButton::Select)),

                    _ => None,
                }
            } else {
                None
            }
        }));

        Subscription::batch(subscriptions)
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Execution control
            Message::Tick(_now) => self.do_tick(),
            Message::TogglePlayback => self.toggle_playback(),
            Message::Step => self.do_step(),
            Message::StepFrame => self.do_step_frame(),
            Message::Reset => self.do_reset(),

            // User interface
            Message::CloseWindow => window::latest().and_then(window::close),
            Message::OpenFile => self.open_file(),

            // Breakpoint management
            Message::BreakpointRemove => self.breakpoint_clear(),
            Message::BreakpointSet(addr) => self.breakpoint_set(addr),
            Message::BreakpointInputChanged(content) => self.breakpoint_update_input(content),

            // Visual components
            Message::ScreenView(msg) => self.screen.update(msg).map(Message::ScreenView),
            Message::MemoryView(msg) => self.view_memory_state.update(msg).map(Message::MemoryView),

            // Machine inputs
            Message::ButtonsPressed(button) => {
                self.machine.button_pressed(button);
                Task::none()
            }
            Message::ButtonsReleased(button) => {
                self.machine.button_released(button);
                Task::none()
            }
        }
    }
    pub fn view(&self) -> Element<'_, Message> {
        let controls = view_control_panel(self.is_running, self);

        let cpu_state = title_panel("CPU", view_cpu::view(self.machine.cpu())).center_x(200);

        let io_registers = title_panel("IO REGISTERS", view_registers::view(&self.machine)).center_x(500);

        let screen = title_panel(
            "SCREEN",
            container(self.screen.view(self.machine.frame()).map(Message::ScreenView))
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

        let content = column![
            controls,
            row![cpu_state, io_registers, screen].spacing(COLUMN_SPACING),
            memory
        ]
        .spacing(COLUMN_SPACING)
        .padding(CONTENT_PADDING);

        Element::from(scrollable(content).direction(Direction::Both {
            vertical: Scrollbar::default(),
            horizontal: Scrollbar::default(),
        }))
    }

    fn do_tick(&mut self) -> Task<Message> {
        let (cycles, break_flag) = self.machine.step_frame().unwrap_or_else(|e| {
            error!("{}", e);
            self.is_running = false;
            (0, false)
        });
        self.total_cycles += cycles as u64;

        if break_flag {
            self.is_running = false;
        }

        self.update(Message::ScreenView(screen::Message::UpdateFrameBuffer))
    }
    fn toggle_playback(&mut self) -> Task<Message> {
        self.is_running = !self.is_running;

        if !self.is_running {
            self.last_update = None;
        }

        Task::none()
    }
    fn do_step(&mut self) -> Task<Message> {
        self.is_running = false;
        self.total_cycles += self.machine.step().expect("Failed to step") as u64;
        Task::none()
    }
    fn do_step_frame(&mut self) -> Task<Message> {
        self.is_running = false;

        let (cycles, _) = self.machine.step_frame().unwrap_or_else(|e| {
            error!("{}", e);
            (0, false)
        });

        self.total_cycles += cycles as u64;
        self.update(Message::ScreenView(screen::Message::UpdateFrameBuffer))
    }
    fn do_reset(&mut self) -> Task<Message> {
        self.machine.reset();
        self.screen.clear();
        self.total_cycles = 0;
        Task::none()
    }
    fn open_file(&mut self) -> Task<Message> {
        let dialog = rfd::FileDialog::new()
            .set_title("Open file")
            .add_filter("Rom", &["gb", "zip"])
            .add_filter("All files", &["*"]);

        if let Some(path) = dialog.pick_file() {
            self.machine.reset();
            self.machine.load_cartridge(path).expect("Failed to load rom");
            self.is_running = true;
        }

        Task::none()
    }
    fn breakpoint_clear(&mut self) -> Task<Message> {
        self.machine.breakpoint_manager_mut().clear();
        Task::none()
    }
    fn breakpoint_set(&mut self, addr: u16) -> Task<Message> {
        self.is_running = true;
        self.machine.breakpoint_manager_mut().add_breakpoint(addr);
        Task::none()
    }
    fn breakpoint_update_input(&mut self, content: String) -> Task<Message> {
        self.breakpoint_at = content;
        Task::none()
    }
}

fn view_control_panel<'a>(is_running: bool, app: &App) -> Element<'a, Message> {
    let run_button = button(if is_running { "Pause" } else { "Play" })
        .width(70)
        .on_press(Message::TogglePlayback)
        .style(button::primary);

    let step_button = button("Step(F7)").on_press(Message::Step).style(button::secondary);

    let reset_button = button("Reset(R)").on_press(Message::Reset).style(button::secondary);

    let step_frame_button = button("Frame(F10)")
        .on_press(Message::StepFrame)
        .style(button::secondary);

    let total_cycles = column![text("cycles:").size(12), text(app.total_cycles).size(12),].align_x(Horizontal::Center);

    let breakpoint_controls = view_breakpoint_controls(app);

    let load_rom = button("Load ROM").style(button::secondary).on_press(Message::OpenFile);

    row![
        run_button,
        step_button,
        step_frame_button,
        reset_button,
        breakpoint_controls,
        load_rom,
        total_cycles,
    ]
    .spacing(BUTTON_SPACING)
    .align_y(Vertical::Center)
    .into()
}

fn view_breakpoint_controls<'a>(app: &App) -> iced::widget::Row<'a, Message> {
    let breakpoint_empty = app.machine.breakpoint_manager().len() == 0;

    let breakpoint_action = || {
        if breakpoint_empty {
            u16::from_str_radix(&app.breakpoint_at, 16)
                .map(Message::BreakpointSet)
                .ok()
        } else {
            Some(Message::BreakpointRemove)
        }
    };

    row![
        text("Breakpoint at: $"),
        text_input("Breakpoint", &app.breakpoint_at)
            .width(60)
            .on_input(Message::BreakpointInputChanged)
            .on_submit_maybe(breakpoint_action()),
        button(if breakpoint_empty { "Go" } else { "Del" })
            .on_press_maybe(breakpoint_action())
            .style(button::secondary),
    ]
    .align_y(Vertical::Center)
}
