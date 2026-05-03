#![recursion_limit = "1024"]

use console_error_panic_hook::set_once as set_panic_hook;
use gbemu_core::{JoypadButton, Machine};
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

const GB_WIDTH: u32 = 160;
const GB_HEIGHT: u32 = 144;
const GB_FRAME_SIZE: usize = (GB_WIDTH * GB_HEIGHT) as usize;

const PALETTE: [(u8, u8, u8); 4] = [
    (155, 188, 15), // 0 - Light
    (139, 172, 15), // 1
    (48, 98, 48),   // 2
    (15, 56, 15),   // 3 - Dark
];

#[wasm_bindgen]
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
impl From<Button> for JoypadButton {
    fn from(btn: Button) -> Self {
        match btn {
            Button::Up => JoypadButton::Up,
            Button::Down => JoypadButton::Down,
            Button::Left => JoypadButton::Left,
            Button::Right => JoypadButton::Right,
            Button::A => JoypadButton::A,
            Button::B => JoypadButton::B,
            Button::Select => JoypadButton::Select,
            Button::Start => JoypadButton::Start,
        }
    }
}

#[wasm_bindgen]
pub struct App {
    machine: Machine,
    ctx: CanvasRenderingContext2d,
    rgba_buffer: Vec<u8>,
}

#[wasm_bindgen]
impl App {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<App, JsError> {
        console_error_panic_hook::set_once();

        let window = window().ok_or_else(|| JsError::new("No window"))?;
        let document = window.document().ok_or_else(|| JsError::new("No document"))?;
        let canvas = document
            .get_element_by_id("screen")
            .ok_or_else(|| JsError::new("Canvas 'screen' not found"))?
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| JsError::new("Element is not a canvas"))?;

        let ctx = canvas
            .get_context("2d")
            .map_err(|_| JsError::new("Failed to get 2d context"))?
            .ok_or_else(|| JsError::new("No 2d context available"))?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| JsError::new("Context is not 2d"))?;
        let rgba_buffer = vec![0u8; GB_FRAME_SIZE * 4];
        let machine = Machine::default();

        Ok(App {
            machine,
            ctx,
            rgba_buffer,
        })
    }

    pub fn load_rom(&mut self, data: &[u8]) -> Result<(), JsError> {
        self.machine
            .load_cartridge_from_bytes(data)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn reset(&mut self) {
        self.machine.reset();
    }

    pub fn step_frame(&mut self) -> Result<(), JsError> {
        self.machine
            .step_frame()
            .map(|_| ())
            .map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn render(&mut self) {
        let frame = self.machine.frame(); // &[u8] - indices 0-3

        // Convertir les indices de couleur en pixels RGBA
        for (i, &index) in frame.iter().enumerate() {
            let color_idx = (index & 0x03) as usize;
            let (r, g, b) = PALETTE[color_idx];

            let base = i * 4;
            self.rgba_buffer[base] = r;
            self.rgba_buffer[base + 1] = g;
            self.rgba_buffer[base + 2] = b;
            self.rgba_buffer[base + 3] = 255; // Alpha opaque
        }

        let image_data =
            ImageData::new_with_u8_clamped_array_and_sh(Clamped(&self.rgba_buffer[..]), GB_WIDTH, GB_HEIGHT)
                .expect("Failed to create ImageData");
        let _ = self.ctx.put_image_data(&image_data, 0.0, 0.0);
    }

    pub fn step_frame_and_render(&mut self) -> Result<(), JsError> {
        self.step_frame()?;
        self.render();
        Ok(())
    }

    pub fn set_button(&mut self, button: Button, pressed: bool) {
        self.machine.button_changed(button.into(), pressed);
    }
}

fn main() {
    set_panic_hook();
}
