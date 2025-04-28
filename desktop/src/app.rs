use crate::style;
use crate::style::text::{reg_flag, reg8};
use crate::theme::color;
use crate::views::view_memory;
use crate::widgets::screen::Screen;
use crate::widgets::{screen, title_panel};
use gbrust_core::{Cpu, CpuFlags, Machine};
use iced::alignment::{Horizontal, Vertical};
use iced::keyboard::key::Named;
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{
    Column, Row, Space, button, column, container, horizontal_space, row, scrollable, text, text_input,
};
use iced::{Color, Element, Fill, Length, Subscription, Task, keyboard, time, window};
use iced_aw::{grid, grid_row};
use log::debug;
use std::time::{Duration, Instant};
use style::text::*;

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
        let controls = view_control(self.is_running, &self);

        let cpu_state = title_panel("CPU", view_cpu_state(&self.machine.cpu())).center_x(200);
        let io_registers = title_panel("IO REGISTERS", view_io_registers(&self.machine)).center_x(500);

        // let listings = view_listings(&self.machine);
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
        u16::from_str_radix(&app.breakpoint_at.as_str(), 16)
            .map(|addr| Message::StepToAddr(addr))
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

fn view_cpu_state<'a>(cpu: &Cpu) -> Element<'a, Message> {
    const SIZE: u16 = 12;

    let reg8 = |name: &'a str, value: u8| -> Element<'a, Message> {
        column![
            row![
                text(name).size(SIZE).style(reg8),
                text("=").size(SIZE),
                text(format!("${:02X}", value)).size(SIZE)
            ]
            .spacing(10),
            row![
                text(format!("{:04b}", value >> 4)).size(SIZE),
                Space::with_width(3.0),
                text(format!("{:04b}", value & 0xF)).size(SIZE)
            ]
        ]
        .align_x(Horizontal::Center)
        .into()
    };
    let reg16 = |name: &'a str, value: u16| -> Element<'a, Message> {
        column![
            row![
                text(name).style(reg16).size(SIZE),
                text("=").size(SIZE),
                text(format!("${:04X}", value)).size(SIZE)
            ]
            .spacing(10),
            row![
                text(format!("{:04b}", (value >> 12) & 0xF)).size(SIZE),
                Space::with_width(3.0),
                text(format!("{:04b}", (value >> 8) & 0xF)).size(SIZE),
                Space::with_width(6.0),
                text(format!("{:04b}", (value >> 4) & 0xF)).size(SIZE),
                Space::with_width(3.0),
                text(format!("{:04b}", value & 0xF)).size(SIZE)
            ]
        ]
        .align_x(Horizontal::Center)
        .into()
    };
    let flags = |name: &'a str, value: bool| -> Element<'a, Message> {
        row![
            text(name).style(reg_flag).size(SIZE),
            text("=").size(SIZE),
            text(if value { "1" } else { "0" }).size(SIZE),
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
        reg16("SP", cpu.sp()),
        reg16("PC", cpu.pc()),
        row![flags("IME", cpu.ime()), flags("HALT", cpu.halt())].spacing(20),
    ]
    .align_x(Horizontal::Center)
    .spacing(6)
    .padding(4)
    .into()
}

#[allow(dead_code)]
fn view_listings<'a>(machine: &Machine) -> Element<'a, Message> {
    const TEXT_SIZE: u16 = 14;
    let instr_row = |addr: u16| -> Element<'a, Message> {
        let mut items: Vec<Element<'a, Message>> = vec![
            text(format!("${addr:04X}"))
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

fn view_io_registers<'a>(machine: &Machine) -> Element<'a, Message> {
    const SIZE: u16 = 12;

    let title = |title: &'a str| -> iced::advanced::widget::Text<'a, _, _> {
        text(format!("{}:", title)).color(color::purple()).size(SIZE)
    };
    let io_reg8 = |name: &'a str, addr: u16, val: u8| -> Element<'a, Message> {
        row![
            Space::with_width(10.0),
            text(format!("${:04X}", addr)).color(color::orange()).size(SIZE),
            Space::with_width(10.0),
            text(name).color(Color::from_rgb8(90, 206, 167)).width(60).size(SIZE),
            text(format!("${:02X}", val)).size(SIZE),
            Space::with_width(10.0),
            text(format!("({:04b}", val >> 4)).size(SIZE),
            Space::with_width(4.0),
            text(format!("{:04b})", val & 0xF)).size(SIZE),
        ]
        .into()
    };
    let io_reg16 = |name: &'a str, addr: u16, val: u16| -> Element<'a, Message> {
        row![
            Space::with_width(10.0),
            text(format!("${:04X}", addr)).color(color::orange()).size(SIZE),
            Space::with_width(10.0),
            text(name).color(Color::from_rgb8(90, 206, 167)).width(60).size(SIZE),
            text(format!("${:04X}", val)).size(SIZE),
        ]
        .into()
    };
    let io_reg_flag = |name: &'a str, val_if, val_ie: bool| -> Element<'a, Message> {
        row![
            Space::with_width(15.0),
            text(name).color(Color::from_rgb8(90, 206, 167)).width(60).size(SIZE),
            if val_if {
                text("On").width(40).color(color::red()).size(SIZE)
            } else {
                text("Off").width(40).color(color::blue()).size(SIZE)
            },
            text("IF").color(color::blue()).size(SIZE),
            Space::with_width(8.0),
            text(if val_if { "1" } else { "0" }).size(SIZE),
            Space::with_width(15.0),
            text("IE").color(color::blue()).size(SIZE),
            Space::with_width(8.0),
            text(if val_ie { "1" } else { "0" }).size(SIZE),
        ]
        .into()
    };

    let ie_val = machine.bus.read_byte(0xFFFF);
    let if_val = machine.bus.read_byte(0xFF0F);
    row![
        column![
            title("INTERRUPTS"),
            io_reg8("IE", 0xFFFF, ie_val),
            io_reg8("IF", 0xFF0F, if_val),
            io_reg_flag("VBLNK", if_val & 0b0000_0001 != 0, ie_val & 0b0000_0001 != 0),
            io_reg_flag("STAT", if_val & 0b0000_0010 != 0, ie_val & 0b0000_0010 != 0),
            io_reg_flag("TIMER", if_val & 0b0000_0100 != 0, ie_val & 0b0000_0100 != 0),
            io_reg_flag("SERIAL", if_val & 0b0000_1000 != 0, ie_val & 0b0000_1000 != 0),
            io_reg_flag("JOYPAD", if_val & 0b0001_0000 != 0, ie_val & 0b0001_0000 != 0),
            title("GBC"),
            io_reg8("KEY1", 0xFF4D, machine.bus.read_byte(0xFF4D)),
            io_reg8("SVBK", 0xFF70, machine.bus.read_byte(0xFF70)),
            title("GBC LCD"),
            io_reg8("BCPS", 0xFF68, machine.bus.read_byte(0xFF68)),
            io_reg8("BCPD", 0xFF69, machine.bus.read_byte(0xFF69)),
            io_reg8("OCPS", 0xFF6A, machine.bus.read_byte(0xFF6A)),
            io_reg8("OCPD", 0xFF6B, machine.bus.read_byte(0xFF6B)),
            io_reg8("VBK", 0xFF4F, machine.bus.read_byte(0xFF4F)),
            title("GBC HDMA"),
            io_reg16("SOURCE", 0xFF51, machine.bus.read_word(0xFF51)),
            io_reg16("DEST", 0xFF52, machine.bus.read_word(0xFF52)),
            title("GBC INFRARED"),
            io_reg8("RP", 0xFF56, machine.bus.read_byte(0xFF56)),
        ]
        .align_x(Horizontal::Left),
        Space::with_width(10.0),
        column![
            title("LCD"),
            io_reg8("LCDC", 0xFF40, machine.bus.read_byte(0xFF40)),
            io_reg8("STAT", 0xFF41, machine.bus.read_byte(0xFF41)),
            io_reg8("SCY", 0xFF42, machine.bus.read_byte(0xFF42)),
            io_reg8("SCX", 0xFF43, machine.bus.read_byte(0xFF43)),
            io_reg8("LY", 0xFF44, machine.bus.read_byte(0xFF44)),
            io_reg8("LYC", 0xFF45, machine.bus.read_byte(0xFF45)),
            io_reg8("DMA", 0xFF46, machine.bus.read_byte(0xFF46)),
            io_reg8("BGP", 0xFF47, machine.bus.read_byte(0xFF47)),
            io_reg8("OBP0", 0xFF48, machine.bus.read_byte(0xFF48)),
            io_reg8("OBP1", 0xFF49, machine.bus.read_byte(0xFF49)),
            io_reg8("WY", 0xFF4A, machine.bus.read_byte(0xFF4A)),
            io_reg8("WX", 0xFF4B, machine.bus.read_byte(0xFF4B)),
            title("TIMER"),
            io_reg8("DIV", 0xFF04, machine.bus.read_byte(0xFF04)),
            io_reg8("TIMA", 0xFF05, machine.bus.read_byte(0xFF05)),
            io_reg8("TMA", 0xFF06, machine.bus.read_byte(0xFF06)),
            io_reg8("TAC", 0xFF07, machine.bus.read_byte(0xFF07)),
            title("INPUT"),
            io_reg8("JOYP", 0xFF00, machine.bus.read_byte(0xFF00)),
            title("SERIAL"),
            io_reg8("SB", 0xFF01, machine.bus.read_byte(0xFF01)),
            io_reg8("SC", 0xFF02, machine.bus.read_byte(0xFF02)),
        ]
        .align_x(Horizontal::Left),
    ]
    .spacing(6)
    .padding(4)
    .into()
}
