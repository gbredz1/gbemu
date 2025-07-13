use crate::app::Message;
use crate::theme::color::{blue, green, orange};
use gbemu_core::{Cpu, CpuFlags};
use iced::Element;
use iced::alignment::Horizontal;
use iced::widget::{Space, row, text};

pub fn view<'a>(cpu: &Cpu) -> Element<'a, Message> {
    const SIZE: u16 = 12;

    let reg8 = |name: &'a str, value: u8| -> Element<'a, Message> {
        iced::widget::column![
            row![
                text(name).size(SIZE).color(orange()),
                text("=").size(SIZE),
                text(format!("${value:02X}")).size(SIZE)
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
        iced::widget::column![
            row![
                text(name).size(SIZE).color(blue()),
                text("=").size(SIZE),
                text(format!("${value:04X}")).size(SIZE)
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
            text(name).size(SIZE).color(green()),
            text("=").size(SIZE),
            text(if value { "1" } else { "0" }).size(SIZE),
        ]
        .spacing(10)
        .into()
    };

    iced::widget::column![
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
