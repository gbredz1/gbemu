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

// Application constants
const DEFAULT_BREAKPOINT: &str = "00e9";
const UPDATE_INTERVAL_MS: u64 = 30;
const BUTTON_SPACING: u16 = 8;
const COLUMN_SPACING: u16 = 10;
const CONTENT_PADDING: u16 = 10;

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

        subscriptions.push(keyboard::on_key_press(|key, _modifiers| match key.as_ref() {
            keyboard::Key::Named(Named::F7) => Some(Message::Step),
            keyboard::Key::Character("r") => Some(Message::Reset),
            keyboard::Key::Named(Named::F10) => Some(Message::StepFrame),
            keyboard::Key::Named(Named::Space) => Some(Message::TogglePlayback),
            keyboard::Key::Named(Named::Escape) => Some(Message::CloseWindow),
            keyboard::Key::Character("l") => Some(Message::OpenFile),
            _ => None,
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
            Message::CloseWindow => window::get_latest().and_then(window::close),
            Message::OpenFile => self.open_file(),

            // Breakpoint management
            Message::BreakpointRemove => self.breakpoint_clear(),
            Message::BreakpointSet(addr) => self.breakpoint_set(addr),
            Message::BreakpointInputChanged(content) => self.breakpoint_update_input(content),

            // Visual components
            Message::ScreenView(msg) => self.screen.update(msg).map(Message::ScreenView),
            Message::MemoryView(msg) => self.view_memory_state.update(msg).map(Message::MemoryView),
        }
    }
    pub fn view(&self) -> Element<Message> {
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
        let (cycles, break_flag) = self.machine.step_frame().expect("Failed to update the machine");
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
        self.total_cycles += self.machine.step_frame().unwrap().0 as u64;
        Task::none()
    }
    fn do_reset(&mut self) -> Task<Message> {
        self.machine.reset();
        self.screen.clear();
        self.total_cycles = 0;
        Task::none()
    }
    fn open_file(&mut self) -> Task<Message> {
        let dialog = native_dialog::DialogBuilder::file()
            .set_title("Open file")
            .add_filter("ROM", ["gb"])
            .open_single_file();

        if let Ok(Some(path)) = dialog.show() {
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

    let total_cycles = text(format!("Cycles: {}", app.total_cycles));

    let breakpoint_controls = view_breakpoint_controls(app);

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
