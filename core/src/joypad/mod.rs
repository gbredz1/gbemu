pub(crate) mod joypad_bus;

use crate::bus::Interrupt;
use crate::joypad::joypad_bus::{JoypadBus, P1JOYP};

#[derive(Default)]
pub struct Joypad {
    buttons: P1JOYP,
    d_pad: P1JOYP,
    prev: P1JOYP,
}

impl Joypad {
    pub fn reset(&mut self, bus: &mut impl JoypadBus) {
        let mut joyp = bus.p1joyp();
        joyp |= P1JOYP::from_bits_truncate(0b0000_1111);
        bus.set_p1joyp(joyp);

        self.buttons |= P1JOYP::all();
        self.d_pad |= P1JOYP::all();
        self.prev = joyp;
    }

    pub fn update(&mut self, bus: &mut impl JoypadBus) {
        let mut joyp = bus.p1joyp();

        joyp |= P1JOYP::from_bits_truncate(0b0000_1111);
        if !joyp.contains(P1JOYP::SELECT_DPAD) {
            joyp &= self.d_pad;
        }
        if !joyp.contains(P1JOYP::SELECT_BUTTONS) {
            joyp &= self.buttons;
        }

        if joyp.bits() & 0x0F != self.prev.bits() & 0x0F {
            bus.set_interrupt_flag(Interrupt::JOYPAD);
        }

        self.prev = joyp;
        bus.set_p1joyp(joyp);
    }
    pub fn button_pressed(&mut self, button: Button) {
        match &button {
            Button::Up | Button::Down | Button::Left | Button::Right => {
                self.d_pad.remove(P1JOYP::from(button));
            }
            Button::A | Button::B | Button::Select | Button::Start => {
                self.buttons.remove(P1JOYP::from(button));
            }
        };
    }
    pub fn button_released(&mut self, button: Button) {
        match &button {
            Button::Up | Button::Down | Button::Left | Button::Right => {
                self.d_pad.insert(P1JOYP::from(button));
            }
            Button::A | Button::B | Button::Select | Button::Start => {
                self.buttons.insert(P1JOYP::from(button));
            }
        };
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub enum Button {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    Select,
    Start,
}
impl From<Button> for P1JOYP {
    fn from(button: Button) -> Self {
        match button {
            Button::Up => P1JOYP::UP,
            Button::Down => P1JOYP::DOWN,
            Button::Left => P1JOYP::LEFT,
            Button::Right => P1JOYP::RIGHT,
            Button::A => P1JOYP::A,
            Button::B => P1JOYP::B,
            Button::Select => P1JOYP::SELECT,
            Button::Start => P1JOYP::START,
        }
    }
}
