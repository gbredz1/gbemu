use crate::app::Message;
use crate::theme::color::*;
use gbrust_core::Machine;
use iced::Element;
use iced::alignment::Horizontal;
use iced::widget::{Space, column, row, text};

pub fn view<'a>(machine: &Machine) -> Element<'a, Message> {
    const SIZE: u16 = 12;

    let title = |title: &'a str| -> iced::advanced::widget::Text<'a, _, _> {
        text(format!("{title}:")).color(purple()).size(SIZE)
    };
    let io_reg8 = |name: &'a str, addr: u16, val: u8| -> Element<'a, Message> {
        row![
            Space::with_width(10.0),
            text(format!("${addr:04X}")).color(orange()).size(SIZE),
            Space::with_width(10.0),
            text(name).color(green()).width(60).size(SIZE),
            text(format!("${val:02X}")).size(SIZE),
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
            text(format!("${addr:04X}")).color(orange()).size(SIZE),
            Space::with_width(10.0),
            text(name).color(orange()).width(60).size(SIZE),
            text(format!("${val:04X}")).size(SIZE),
        ]
        .into()
    };
    let io_reg_flag = |name: &'a str, val_if, val_ie: bool| -> Element<'a, Message> {
        row![
            Space::with_width(15.0),
            text(name).color(orange()).width(60).size(SIZE),
            if val_if {
                text("On").width(40).color(red()).size(SIZE)
            } else {
                text("Off").width(40).color(blue()).size(SIZE)
            },
            text("IF").color(blue()).size(SIZE),
            Space::with_width(8.0),
            text(if val_if { "1" } else { "0" }).size(SIZE),
            Space::with_width(15.0),
            text("IE").color(blue()).size(SIZE),
            Space::with_width(8.0),
            text(if val_ie { "1" } else { "0" }).size(SIZE),
        ]
        .into()
    };

    let ie_val = machine.bus().read_byte(0xFFFF);
    let if_val = machine.bus().read_byte(0xFF0F);
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
            io_reg8("KEY1", 0xFF4D, machine.bus().read_byte(0xFF4D)),
            io_reg8("SVBK", 0xFF70, machine.bus().read_byte(0xFF70)),
            title("GBC LCD"),
            io_reg8("BCPS", 0xFF68, machine.bus().read_byte(0xFF68)),
            io_reg8("BCPD", 0xFF69, machine.bus().read_byte(0xFF69)),
            io_reg8("OCPS", 0xFF6A, machine.bus().read_byte(0xFF6A)),
            io_reg8("OCPD", 0xFF6B, machine.bus().read_byte(0xFF6B)),
            io_reg8("VBK", 0xFF4F, machine.bus().read_byte(0xFF4F)),
            title("GBC HDMA"),
            io_reg16("SOURCE", 0xFF51, machine.bus().read_word(0xFF51)),
            io_reg16("DEST", 0xFF52, machine.bus().read_word(0xFF52)),
            title("GBC INFRARED"),
            io_reg8("RP", 0xFF56, machine.bus().read_byte(0xFF56)),
        ]
        .align_x(Horizontal::Left),
        Space::with_width(10.0),
        column![
            title("LCD"),
            io_reg8("LCDC", 0xFF40, machine.bus().read_byte(0xFF40)),
            io_reg8("STAT", 0xFF41, machine.bus().read_byte(0xFF41)),
            io_reg8("SCY", 0xFF42, machine.bus().read_byte(0xFF42)),
            io_reg8("SCX", 0xFF43, machine.bus().read_byte(0xFF43)),
            io_reg8("LY", 0xFF44, machine.bus().read_byte(0xFF44)),
            io_reg8("LYC", 0xFF45, machine.bus().read_byte(0xFF45)),
            io_reg8("DMA", 0xFF46, machine.bus().read_byte(0xFF46)),
            io_reg8("BGP", 0xFF47, machine.bus().read_byte(0xFF47)),
            io_reg8("OBP0", 0xFF48, machine.bus().read_byte(0xFF48)),
            io_reg8("OBP1", 0xFF49, machine.bus().read_byte(0xFF49)),
            io_reg8("WY", 0xFF4A, machine.bus().read_byte(0xFF4A)),
            io_reg8("WX", 0xFF4B, machine.bus().read_byte(0xFF4B)),
            title("TIMER"),
            io_reg8("DIV", 0xFF04, machine.bus().read_byte(0xFF04)),
            io_reg8("TIMA", 0xFF05, machine.bus().read_byte(0xFF05)),
            io_reg8("TMA", 0xFF06, machine.bus().read_byte(0xFF06)),
            io_reg8("TAC", 0xFF07, machine.bus().read_byte(0xFF07)),
            title("INPUT"),
            io_reg8("JOYP", 0xFF00, machine.bus().read_byte(0xFF00)),
            title("SERIAL"),
            io_reg8("SB", 0xFF01, machine.bus().read_byte(0xFF01)),
            io_reg8("SC", 0xFF02, machine.bus().read_byte(0xFF02)),
        ]
        .align_x(Horizontal::Left),
    ]
    .spacing(6)
    .padding(4)
    .into()
}
