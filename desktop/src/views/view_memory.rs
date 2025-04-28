use gbrust_core::Machine;

use crate::theme::color::{green, orange, pink, purple, yellow};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{Row, Space, button, column, container, horizontal_space, row, text, text_input};
use iced::{Element, Fill, Task};
use iced_aw::{grid, grid_row};

pub struct State {
    input_string: String,
    addr_start: u16,
}

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    Update(u16),
    Increment(u8),
    Decrement(u8),
}

const MAX_ADDR: u16 = 0xFF0;

impl Default for State {
    fn default() -> Self {
        Self {
            input_string: "000".to_string(),
            addr_start: 0,
        }
    }
}

impl State {
    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::InputChanged(addr) => {
                let addr = addr.chars().filter(|c| c.is_ascii_hexdigit()).collect();
                self.input_string = addr;

                match u16::from_str_radix(&self.input_string, 16) {
                    Ok(addr) => match addr {
                        0..=MAX_ADDR => self.update(Message::Update(addr)),
                        _ => {
                            self.input_string = format!("{MAX_ADDR:X}");
                            self.update(Message::Update(MAX_ADDR))
                        }
                    },
                    _ => Task::none(),
                }
            }
            Message::Increment(val) => {
                let res = match self.addr_start.wrapping_add(val as u16) {
                    res if res < self.addr_start => MAX_ADDR,
                    res if res > MAX_ADDR => MAX_ADDR,
                    res => res,
                };

                if self.addr_start != res && res <= MAX_ADDR {
                    self.addr_start = res;
                    self.input_string = format!("{:03X}", self.addr_start);
                }
                Task::none()
            }

            Message::Decrement(val) => {
                let res = match self.addr_start.wrapping_sub(val as u16) {
                    res if res > self.addr_start => 0x0,
                    res => res,
                };

                if self.addr_start != res && res < self.addr_start {
                    self.addr_start = res;
                    self.input_string = format!("{:03X}", self.addr_start);
                }
                Task::none()
            }
            Message::Update(addr) => {
                self.addr_start = addr;
                Task::none()
            }
        }
    }
}

macro_rules! memory_row {
    ($f:expr,
        $v0:expr, $v1:expr, $v2:expr, $v3:expr,
        $v4:expr, $v5:expr, $v6:expr, $v7:expr) => {
        row![
            $f($v0),
            Space::with_width(SPACE_BYTE),
            $f($v1),
            Space::with_width(SPACE_BYTE),
            $f($v2),
            Space::with_width(SPACE_BYTE),
            $f($v3),
            Space::with_width(SPACE_BYTE_4),
            $f($v4),
            Space::with_width(SPACE_BYTE),
            $f($v5),
            Space::with_width(SPACE_BYTE),
            $f($v6),
            Space::with_width(SPACE_BYTE),
            $f($v7),
        ]
    };
}
macro_rules! memory_row_addr {
    ($f:expr, $addr:expr) => {
        memory_row!(
            $f,
            $addr + 0,
            $addr + 1,
            $addr + 2,
            $addr + 3,
            $addr + 4,
            $addr + 5,
            $addr + 6,
            $addr + 7
        )
    };
}

pub fn view<'a>(state: &State, machine: &Machine) -> Element<'a, Message> {
    const SIZE: u16 = 12;
    const SPACE_BYTE: u16 = 4; // macro
    const SPACE_BYTE_4: u16 = 7; // macro

    const ADDR_COUNT: usize = 16;

    let button_decrement =
        button(text("<").size(SIZE))
            .style(button::secondary)
            .on_press_maybe(match state.addr_start {
                val if val > 0 => Some(Message::Decrement(1)),
                _ => None,
            });
    let button_decrement10 =
        button(text("<<").size(SIZE))
            .style(button::secondary)
            .on_press_maybe(match state.addr_start {
                val if val > 0 => Some(Message::Decrement(0x10)),
                _ => None,
            });
    let button_increment =
        button(text(">").size(SIZE))
            .style(button::secondary)
            .on_press_maybe(match state.addr_start {
                val if val < MAX_ADDR => Some(Message::Increment(1)),
                _ => None,
            });
    let button_increment10 =
        button(text(">>").size(SIZE))
            .style(button::secondary)
            .on_press_maybe(match state.addr_start {
                val if val < MAX_ADDR => Some(Message::Increment(0x10)),
                _ => None,
            });

    let controls = row![
        text("Start at: ").size(SIZE),
        button_decrement10,
        button_decrement,
        text_input("start", &state.input_string)
            .size(SIZE)
            .align_x(Horizontal::Right)
            .width(50)
            .on_input(Message::InputChanged),
        button_increment,
        button_increment10,
        horizontal_space(),
        row![
            button(text("SP").size(SIZE).color(pink()))
                .style(button::text)
                .on_press(Message::InputChanged(format!("{:03X}", machine.cpu().sp() / 0x10))),
            button(text("PC").size(SIZE).color(purple()))
                .style(button::text)
                .on_press(Message::InputChanged(format!("{:03X}", machine.cpu().pc() / 0x10))),
            button(text("HL").size(SIZE).color(yellow()))
                .style(button::text)
                .on_press(Message::InputChanged(format!("{:03X}", machine.cpu().hl() / 0x10))),
            Row::from_vec(
                [
                    MemorySectors::RomBank0,
                    MemorySectors::RomBank1,
                    MemorySectors::VideoRam
                ]
                .iter()
                .map(|sector| {
                    button(text(sector.name()).size(SIZE))
                        .style(button::text)
                        .on_press(Message::InputChanged(sector.addr().into()))
                        .into()
                })
                .collect()
            ),
        ],
        horizontal_space(),
    ]
    .align_y(Vertical::Center);

    let mem_header = |value: &'a str| text(value).size(SIZE).color(green());

    let header = row![
        text("Address").size(SIZE).width(60).color(purple()),
        memory_row!(mem_header, "00", "01", "02", "03", "04", "05", "06", "07"),
        memory_row!(mem_header, "08", "09", "0A", "0B", "0C", "0D", "0E", "0F"),
    ]
    .spacing(10);

    let mut grid = grid!();

    let offset = state.addr_start as usize;
    let range: Vec<usize> = (offset..)
        .take_while(|&x| x <= 0xFFF)
        .map(|i| i * 0x10)
        .take(ADDR_COUNT)
        .collect();

    let mem_byte = |addr: u16| {
        let value = machine.bus.read_byte(addr);

        let t = text(format!("{value:02x}")).size(SIZE);
        match addr {
            addr if addr == machine.cpu().sp() => t.color(pink()),
            addr if addr == machine.cpu().pc() => t.color(purple()),
            addr if addr == machine.cpu().hl() => t.color(yellow()),
            _ => t,
        }
    };

    let mem_ascii = |addr: u16| -> Element<'a, Message> {
        let value = match machine.bus.read_byte(addr) {
            val if (0x20..=0xFE).contains(&val) => val as char,
            _ => '.',
        };
        text(format!("{value}")).size(SIZE).into()
    };

    for addr in range {
        let addr = addr as u16;
        grid = grid.push(grid_row!(
            horizontal_space(),
            text(format!("${addr:04X}")).size(SIZE).width(50).color(orange()),
            memory_row_addr!(mem_byte, addr),
            memory_row_addr!(mem_byte, addr + 8),
            Row::from_vec((0..=0xF).map(|i| mem_ascii(addr.wrapping_add(i))).collect()),
        ));
    }

    grid = grid.column_spacing(10);

    let content = container(column![header, grid]).width(Fill);

    column![controls, content].spacing(10).padding(8).into()
}


#[allow(dead_code)]
enum MemorySectors {
    RomBank0,
    RomBank1,
    VideoRam,
    ExternalRam,
    WorkRam0,
    WorkRamN,
    EchoRam,
    Oam,
    Unusable,
    IORegister,
    HighRam,
}

impl MemorySectors {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            MemorySectors::RomBank0 => "ROM0",
            MemorySectors::RomBank1 => "ROM1",
            MemorySectors::VideoRam => "VRAM",
            MemorySectors::ExternalRam => "SRAM",
            MemorySectors::WorkRam0 => "WRA0",
            MemorySectors::WorkRamN => "WRAN",
            MemorySectors::EchoRam => "ECHO",
            MemorySectors::Oam => "OAM",
            MemorySectors::Unusable => "---",
            MemorySectors::IORegister => "I/O",
            MemorySectors::HighRam => "HRAM",
        }
    }
    fn addr(&self) -> &'static str {
        match self {
            MemorySectors::RomBank0 => "000",
            MemorySectors::RomBank1 => "400",
            MemorySectors::VideoRam => "800",
            MemorySectors::ExternalRam => "A00",
            MemorySectors::WorkRam0 => "CFF",
            MemorySectors::WorkRamN => "DFF",
            MemorySectors::EchoRam => "E00",
            MemorySectors::Oam => "FE0",
            MemorySectors::Unusable => "FEA",
            MemorySectors::IORegister => "FF0",
            MemorySectors::HighRam => "FF8",
        }
    }
}
