use crate::widgets::screen;
use crate::widgets::screen::Screen;
use gbrust_core::{Cpu, CpuFlags, Machine};
use iced::advanced::Widget;
use iced::alignment::Horizontal;
use iced::widget::{Column, Row, Space, button, column, container, row, scrollable, text};
use iced::{Color, Element, Fill, Length, Subscription, border, time};
use std::time::{Duration, Instant};

pub(crate) struct App {
    machine: Machine,
    last_update: Option<Instant>,
    is_running: bool,
    screen: Screen,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick(Instant),
    TogglePlayback,
    StepOver,
    Reset,
    Screen(screen::Message),
}

impl Default for App {
    fn default() -> Self {
        let mut device = Machine::default();
        device
            .load_cartridge("roms/tetris.gb")
            .expect("Failed to load cartridge");

        Self {
            machine: device,
            last_update: None,
            is_running: false,
            screen: Screen::new(),
        }
    }
}

impl App {
    pub fn title(&self) -> String {
        String::from("My App")
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if self.is_running {
            time::every(Duration::from_millis(17)).map(|_| Message::Tick(Instant::now()))
        } else {
            Subscription::none()
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Tick(now) => {
                let last_update = self.last_update.unwrap_or(now);
                let delta = now - last_update;
                self.last_update = Some(now);

                self.machine.update(&delta).expect("Failed to update machine");

                // if vblank occured or simple 60Hz update
                let frame_buffer = self.machine.frame().clone();
                self.update(Message::Screen(screen::Message::UpdateFrameBuffer(frame_buffer)))
            }
            Message::StepOver => {
                self.is_running = false;
                self.last_update = None;

                self.machine.step().expect("Failed to step");
                // if vblank occured or simple 60Hz update
                let frame_buffer = self.machine.frame().clone();
                self.update(Message::Screen(screen::Message::UpdateFrameBuffer(frame_buffer)))
            }

            Message::TogglePlayback => {
                self.is_running = !self.is_running;
                if !self.is_running {
                    self.last_update = None;
                }
            }
            Message::Reset => {
                self.machine.reset();
                self.screen.clear();
            }

            Message::Screen(msg) => {
                self.screen.update(msg);
            }
        }
    }
    pub fn view(&self) -> Element<Message> {
        let controls = view_control(self.is_running);

        let cpu_state = container(
            column![
                container(text("CPU State").center().width(Fill)).style(|_| {
                    container::Style::default()
                        .background(Color::from_rgba8(27, 161, 56, 0.1))
                        .border(border::color(Color::WHITE).width(1))
                }),
                view_cpu_state(&self.machine.cpu()),
            ]
            .align_x(Horizontal::Center),
        )
        .style(|_| {
            container::Style::default()
                .background(Color::from_rgba8(0xC, 0xC, 0xC, 0.5))
                .border(border::color(Color::WHITE).width(1))
        })
        .center_x(200);

        let listings = view_listings(&self.machine);

        let screen = self.screen.view().map(move |msg| Message::Screen(msg));

        let content = column![controls, row![cpu_state, listings, screen]];
        Element::from(content).explain(Color::from_rgb8(252, 15, 192))
    }
}

fn view_control<'a>(is_running: bool) -> Element<'a, Message> {
    let run_button = button(if is_running { "Pause" } else { "Play" })
        .on_press(Message::TogglePlayback)
        .style(button::primary);
    let step_button = button("Step over").on_press(Message::StepOver).style(button::secondary);
    let reset_button = button("Reset").on_press(Message::Reset).style(button::secondary);

    row![run_button, step_button, reset_button].into()
}

fn view_cpu_state<'a>(cpu: &Cpu) -> Element<'a, Message> {
    let reg8 = |name: &'a str, value: u8| -> Element<'a, Message> {
        column![
            row![
                text(name).color(Color::from_rgb8(90, 206, 167)),
                text("="),
                text(format!("${:02X}", value))
            ]
            .spacing(10),
            row![
                text(format!("{:04b}", value >> 4)),
                text(format!("{:04b}", value & 0b0000_1111))
            ]
            .spacing(10),
        ]
        .align_x(Horizontal::Center)
        .into()
    };
    let reg16 = |name: &'a str, value: u16| -> Element<'a, Message> {
        column![
            row![
                text(name).color(Color::from_rgb8(255, 211, 0)),
                text("="),
                text(format!("${:04X}", value))
            ]
            .spacing(10),
            row![
                text(format!("{:04b}", (value >> 12) & 0b0000_1111)),
                text(format!("{:04b}", (value >> 8) & 0b0000_1111)),
                text(format!("{:04b}", (value >> 4) & 0b0000_1111)),
                text(format!("{:04b}", value & 0b0000_1111))
            ]
            .spacing(10),
        ]
        .align_x(Horizontal::Center)
        .into()
    };
    let flags = |name: &'a str, value: bool| -> Element<'a, Message> {
        row![
            text(name).color(Color::from_rgb8(223, 165, 72)),
            text("="),
            text(if value { "1" } else { "0" }),
        ]
        .spacing(10)
        .into()
    };

    column![
        row![flags("Z", cpu.flag(CpuFlags::Z)), flags("N", cpu.flag(CpuFlags::N))].spacing(20),
        row![flags("H", cpu.flag(CpuFlags::H)), flags("C", cpu.flag(CpuFlags::C))].spacing(20),
        row![reg8("A", cpu.a()), reg8("F", cpu.f())].spacing(10),
        row![reg8("B", cpu.b()), reg8("C", cpu.c())].spacing(10),
        row![reg8("D", cpu.d()), reg8("E", cpu.e())].spacing(10),
        row![reg8("H", cpu.h()), reg8("L", cpu.l())].spacing(10),
        reg16("PC", cpu.pc()),
        reg16("SP", cpu.sp()),
        row![flags("IME", cpu.ime()), flags("HALT", cpu.halt())].spacing(20),
    ]
    .align_x(Horizontal::Center)
    .spacing(6)
    .padding(4)
    .into()
}

fn view_listings<'a>(machine: &Machine) -> Element<'a, Message> {
    const TEXT_SIZE: u16 = 14;
    let instr_row = |addr: u16| -> Element<'a, Message> {
        let mut items: Vec<Element<'a, Message>> = vec![
            text(format!("${:04X}", addr))
                .color(Color::from_rgb8(255, 211, 0))
                .size(TEXT_SIZE)
                .into(),
            text("87").color(Color::from_rgb8(50, 211, 0)).size(TEXT_SIZE).into(),
            Space::with_width(Length::Fixed(50.0)).into(),
        ];
        if machine.cpu().pc() == addr {
            items.push(text(">").color(Color::from_rgb8(255, 50, 50)).size(TEXT_SIZE).into());
        }
        items.push(text("ADD A,A").size(TEXT_SIZE).into());

        Row::with_children(items).spacing(10).width(Fill).into()
    };

    let mut lines = vec![];
    for i in 0x0000..0x0100 {
        lines.push(instr_row(i));
    }

    let listing = Column::with_children(lines.into_iter().map(Element::from));

    scrollable(listing).height(800).width(400).into()
}
