use gbrust_core::Machine;

use crate::theme::color::{green, orange, purple};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, horizontal_space, row, text, text_input};
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

pub fn view<'a>(state: &State, machine: &Machine) -> Element<'a, Message> {
    const SIZE: u16 = 12;

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
    ]
    .align_y(Vertical::Center);

    let header = row![
        text("Address").size(SIZE).width(60).color(purple()),
        text("00 01 02 03").color(green()).size(SIZE),
        text("04 05 06 07").color(green()).size(SIZE),
        text("08 09 0A 0B").color(green()).size(SIZE),
        text("0C 0D 0E 0F").color(green()).size(SIZE),
    ]
    .spacing(10);

    let mut grid = grid!();

    let offset = state.addr_start as usize;
    let range: Vec<usize> = (offset..)
        .take_while(|&x| x <= 0xFFF)
        .map(|i| i * 0x10)
        .take(ADDR_COUNT)
        .collect();

    for addr in range {
        let addr = addr as u16;
        grid = grid.push(grid_row!(
            horizontal_space(),
            text(format!("${addr:04X}")).size(SIZE).width(50).color(orange()),
            text(format!(
                "{:02x} {:02x} {:02x} {:02x}",
                machine.bus.read_byte(addr),
                machine.bus.read_byte(addr + 0x1),
                machine.bus.read_byte(addr + 0x2),
                machine.bus.read_byte(addr + 0x3)
            ))
            .size(SIZE),
            text(format!(
                "{:02x} {:02x} {:02x} {:02x}",
                machine.bus.read_byte(addr + 0x4),
                machine.bus.read_byte(addr + 0x5),
                machine.bus.read_byte(addr + 0x6),
                machine.bus.read_byte(addr + 0x7)
            ))
            .size(SIZE),
            text(format!(
                "{:02x} {:02x} {:02x} {:02x}",
                machine.bus.read_byte(addr + 0x8),
                machine.bus.read_byte(addr + 0x9),
                machine.bus.read_byte(addr + 0xA),
                machine.bus.read_byte(addr + 0xB)
            ))
            .size(SIZE),
            text(format!(
                "{:02x} {:02x} {:02x} {:02x}",
                machine.bus.read_byte(addr + 0xC),
                machine.bus.read_byte(addr + 0xD),
                machine.bus.read_byte(addr + 0xE),
                machine.bus.read_byte(addr + 0xF)
            ))
            .size(SIZE),
        ));
    }

    grid = grid.column_spacing(10);

    let content = container(column![header, grid]).width(Fill);

    column![controls, content].spacing(10).padding(8).into()
}
