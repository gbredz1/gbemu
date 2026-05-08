use crate::bus::Interrupt;
use crate::ppu::mode::Mode;
pub(crate) use crate::ppu::ppu_bus::PpuBus;
pub(crate) use crate::ppu::ppu_bus::{LcdControl, LcdStatus};
use crate::ppu::sprite::Sprite;
use std::collections::VecDeque;

mod mode;
mod ppu_bus;
mod sprite;

const LCD_WIDTH: u8 = 160;
const LCD_HEIGHT: u8 = 144;
const DOTS_PER_LINE: u32 = 456;
const LINES_PER_FRAME: u8 = 154;
const OAM_SCAN_DOTS: u32 = 80;

/// Pixel pushed to the OBJ FIFO.
#[derive(Clone, Copy, Default)]
struct ObjPixel {
    color: u8,        // 0..3 (0 = transparent for OBJ)
    palette_obp1: bool,
    bg_priority: bool,
}

/// State machine of the background/window pixel fetcher.
#[derive(Clone, Copy, PartialEq, Eq)]
enum FetchStep {
    GetTile,
    GetTileLow,
    GetTileHigh,
    Sleep,
    Push,
}

/// Internal state used for an OBJ fetch. Suspends the BG fetcher while active.
struct ObjFetch {
    sprite_idx: usize, // index in `line_sprites`
    step: FetchStep,
    sub: u8, // sub-step counter (each step takes 2 dots)
    tile_low: u8,
    tile_high: u8,
}

pub(crate) struct Ppu {
    /// Position in the current scanline, 0..456 (T-cycles aka dots).
    dot: u32,
    /// Sprites selected during OAM scan for the current scanline (max 10).
    line_sprites: Vec<Sprite>,
    /// Sprites already consumed (rendered) for the current line (indices into line_sprites).
    sprite_done: [bool; 10],

    // --- Background / window fetcher state ---
    fetch_step: FetchStep,
    fetch_sub: u8, // each visible step takes 2 dots
    fetch_x: u8,   // tile column counter (in tiles) for the current fetch line
    fetch_tile_id: u8,
    fetch_tile_low: u8,
    fetch_tile_high: u8,
    /// Whether the BG fetcher is currently fetching window tiles.
    fetching_window: bool,
    /// Internal window line counter (only increments on lines that drew window).
    window_line: u8,
    /// Whether the window has been triggered at least once this frame
    /// (i.e. WY == LY happened on a previous or current line while window was visible).
    wy_triggered: bool,
    /// Whether the window already activated on this current line.
    window_active_on_line: bool,

    // --- BG/OBJ FIFOs ---
    bg_fifo: VecDeque<u8>,        // 0..3 color indexes (BG/Win)
    obj_fifo: VecDeque<ObjPixel>, // up to 8 entries

    /// Currently-active OBJ fetch (None when BG fetcher runs).
    obj_fetch: Option<ObjFetch>,

    /// Number of pixels emitted on the current scanline (0..160).
    lx: u8,
    /// Pixels left to discard at the start of the line because of SCX % 8.
    scx_discard: u8,
    /// True while we are in the drawing phase of the current line.
    drawing: bool,
    /// Cached previous STAT IRQ line level for edge detection.
    prev_stat_line: bool,
    /// LCD enabled latch (used to handle re-enabling).
    lcd_was_enabled: bool,

    // buffer
    pub frame_buffer: [u8; LCD_WIDTH as usize * LCD_HEIGHT as usize],
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            dot: 0,
            line_sprites: Vec::with_capacity(10),
            sprite_done: [false; 10],
            fetch_step: FetchStep::GetTile,
            fetch_sub: 0,
            fetch_x: 0,
            fetch_tile_id: 0,
            fetch_tile_low: 0,
            fetch_tile_high: 0,
            fetching_window: false,
            window_line: 0,
            wy_triggered: false,
            window_active_on_line: false,
            bg_fifo: VecDeque::with_capacity(16),
            obj_fifo: VecDeque::with_capacity(8),
            obj_fetch: None,
            lx: 0,
            scx_discard: 0,
            drawing: false,
            prev_stat_line: false,
            lcd_was_enabled: false,
            frame_buffer: [0; LCD_WIDTH as usize * LCD_HEIGHT as usize],
        }
    }
}

impl Ppu {
    pub fn reset(&mut self, bus: &mut impl PpuBus) {
        *self = Self::default();
        bus.write_mode(Mode::OAMScan);

        // ly and lyc can update LCDC
        bus.set_ly(0);
        bus.set_lyc(0);

        bus.set_lcdc_u8(0x91);
        bus.set_stat_u8(0x80);
        bus.set_scy(0);
        bus.set_scx(0);
        bus.set_dma_u8(0xFF);
        bus.set_bgp(0xFC);
        bus.set_obp0(0xFF);
        bus.set_obp1(0xFF);
        bus.set_wy(0);
        bus.set_wx(0);

        for addr in 0xFE00..0xFEA0 {
            bus.write_internal_byte(addr, 0);
        }

        self.lcd_was_enabled = bus.lcdc().contains(LcdControl::ENABLE);
    }

    /// Advance the PPU by `cycles` T-cycles (dots).
    pub fn update(&mut self, bus: &mut impl PpuBus, cycles: u32) {
        let lcd_on = bus.lcdc().contains(LcdControl::ENABLE);
        if !lcd_on {
            if self.lcd_was_enabled {
                // LCD turned off: reset internal state, blank the screen, force HBlank/LY=0.
                self.dot = 0;
                self.lx = 0;
                self.drawing = false;
                self.bg_fifo.clear();
                self.obj_fifo.clear();
                self.obj_fetch = None;
                self.line_sprites.clear();
                self.sprite_done = [false; 10];
                self.fetching_window = false;
                self.window_active_on_line = false;
                self.wy_triggered = false;
                self.window_line = 0;
                self.frame_buffer.fill(0);
                bus.set_ly(0);
                bus.write_mode(Mode::HBlank);
                self.prev_stat_line = false;
            }
            self.lcd_was_enabled = false;
            return;
        }
        self.lcd_was_enabled = true;

        for _ in 0..cycles {
            self.tick(bus);
        }
    }

    /// One PPU dot.
    fn tick(&mut self, bus: &mut impl PpuBus) {
        let line = bus.ly();

        // Detect start-of-line transitions.
        if self.dot == 0 {
            // Window WY == LY check happens every line at the start of mode 2.
            if line < LCD_HEIGHT && line == bus.wy() {
                self.wy_triggered = true;
            }
            self.window_active_on_line = false;

            if line < LCD_HEIGHT {
                bus.write_mode(Mode::OAMScan);
                self.line_sprites.clear();
                self.sprite_done = [false; 10];
            } else if line == LCD_HEIGHT {
                bus.write_mode(Mode::VBlank);
                bus.update_interrupt_flag(Interrupt::VBLANK, true);
                // Reset window line counter at frame start (after VBlank).
            }
            self.update_lyc_flag(bus);
        }

        if line < LCD_HEIGHT {
            if self.dot < OAM_SCAN_DOTS {
                // Mode 2: OAM scan. Perform all selection at dot 0 for simplicity.
                if self.dot == 0 {
                    self.scan_oam(bus, line);
                }
            } else if !self.drawing && self.dot == OAM_SCAN_DOTS {
                // Enter Mode 3 (Drawing).
                self.start_drawing(bus);
                self.drawing = true;
                bus.write_mode(Mode::PixelTransfer);
            }

            if self.drawing {
                self.drawing_tick(bus, line);
                if self.lx >= LCD_WIDTH {
                    // End of drawing → HBlank.
                    self.drawing = false;
                    bus.write_mode(Mode::HBlank);
                    if self.window_active_on_line {
                        self.window_line = self.window_line.wrapping_add(1);
                    }
                }
            }
        }

        self.update_stat_irq(bus);

        // Advance to next dot/line.
        self.dot += 1;
        if self.dot >= DOTS_PER_LINE {
            self.dot = 0;
            let new_ly = (line + 1) % LINES_PER_FRAME;
            bus.set_ly(new_ly);
            if new_ly == 0 {
                // New frame: reset frame-scoped trackers.
                self.wy_triggered = false;
                self.window_line = 0;
            }
        }
    }

    fn update_lyc_flag(&self, bus: &mut impl PpuBus) {
        let eq = bus.ly() == bus.lyc();
        bus.update_stat(LcdStatus::LYC_EQUAL, eq);
    }

    fn update_stat_irq(&mut self, bus: &mut impl PpuBus) {
        let stat = bus.stat();
        let mode = bus.read_mode();
        let mut line = false;
        if stat.contains(LcdStatus::LYC_INTERRUPT) && stat.contains(LcdStatus::LYC_EQUAL) {
            line = true;
        }
        match mode {
            Mode::HBlank => {
                if stat.contains(LcdStatus::HBLANK_INTERRUPT) {
                    line = true;
                }
            }
            Mode::VBlank => {
                if stat.contains(LcdStatus::VBLANK_INTERRUPT) {
                    line = true;
                }
            }
            Mode::OAMScan => {
                if stat.contains(LcdStatus::OAM_INTERRUPT) {
                    line = true;
                }
            }
            Mode::PixelTransfer => {}
        }
        if line && !self.prev_stat_line {
            bus.update_interrupt_flag(Interrupt::LCD_STAT, true);
        }
        self.prev_stat_line = line;
    }

    /// Mode 2: scan all 40 sprites and pick up to 10 visible ones for `line`.
    fn scan_oam(&mut self, bus: &impl PpuBus, line: u8) {
        self.line_sprites.clear();
        let double_height = bus.lcdc().contains(LcdControl::OBJ_SIZE);
        let height: i16 = if double_height { 16 } else { 8 };
        let line_i = line as i16;

        for sprite_idx in (0..40 * 4).step_by(4) {
            let bytes = [
                bus.read_oam(sprite_idx),
                bus.read_oam(sprite_idx + 1),
                bus.read_oam(sprite_idx + 2),
                bus.read_oam(sprite_idx + 3),
            ];
            let sprite = Sprite::from(bytes);
            // Visible if line is within [y, y + height). X is irrelevant for OAM scan.
            if line_i >= sprite.y() && line_i < sprite.y() + height {
                self.line_sprites.push(sprite);
                if self.line_sprites.len() >= 10 {
                    break;
                }
            }
        }
        // Note: we keep OAM order in the buffer. Trigger order is determined by X
        // when iterating, which gives correct DMG priority (lower X first, OAM
        // index breaks ties due to the stable scan order).
    }

    fn start_drawing(&mut self, bus: &impl PpuBus) {
        self.lx = 0;
        self.bg_fifo.clear();
        self.obj_fifo.clear();
        self.obj_fetch = None;
        self.fetch_step = FetchStep::GetTile;
        self.fetch_sub = 0;
        self.fetch_x = 0;
        self.fetching_window = false;
        self.scx_discard = bus.scx() % 8;
    }

    fn drawing_tick(&mut self, bus: &impl PpuBus, line: u8) {
        // 1. Maybe trigger an OBJ fetch (only if BG FIFO already has a pixel ready
        //    and OBJ rendering is enabled; the BG fetcher pauses while OBJ fetches).
        if self.obj_fetch.is_none() && bus.lcdc().contains(LcdControl::OBJ_ENABLE) {
            if let Some(idx) = self.find_sprite_at(self.lx) {
                self.obj_fetch = Some(ObjFetch {
                    sprite_idx: idx,
                    step: FetchStep::GetTile,
                    sub: 0,
                    tile_low: 0,
                    tile_high: 0,
                });
                self.sprite_done[idx] = true;
            }
        }

        // 2. Check window trigger before fetching.
        self.maybe_trigger_window(bus, line);

        // 3. Tick fetchers (OBJ has priority over BG).
        if self.obj_fetch.is_some() {
            self.tick_obj_fetch(bus, line);
        } else {
            self.tick_bg_fetch(bus, line);
        }

        // 4. Try to push a pixel to the LCD.
        if self.obj_fetch.is_none() {
            self.try_push_pixel(bus, line);
        }
    }

    fn maybe_trigger_window(&mut self, bus: &impl PpuBus, _line: u8) {
        if self.fetching_window {
            return;
        }
        if !bus.lcdc().contains(LcdControl::WINDOW_ENABLE) {
            return;
        }
        if !self.wy_triggered {
            return;
        }
        let wx = bus.wx();
        // Effective window start X is wx - 7.
        // Trigger when the next pixel to be emitted reaches WX - 7.
        let target = wx as i16 - 7;
        if self.lx as i16 == target.max(0) && (wx as i16 + (self.lx as i16 - target).max(0)) >= 0 {
            // Switch fetcher to window mode.
            self.fetching_window = true;
            self.window_active_on_line = true;
            self.bg_fifo.clear();
            self.fetch_step = FetchStep::GetTile;
            self.fetch_sub = 0;
            self.fetch_x = 0;
            // OBJ FIFO is intentionally NOT cleared.
        }
    }

    /// Returns the index of the first pending sprite whose effective X equals lx.
    /// We pick the sprite with the lowest X first; OAM order breaks ties.
    fn find_sprite_at(&self, lx: u8) -> Option<usize> {
        let mut best: Option<usize> = None;
        let mut best_x: i16 = i16::MAX;
        for (i, s) in self.line_sprites.iter().enumerate() {
            if self.sprite_done[i] {
                continue;
            }
            let sx = s.x();
            // Sprite is "ready" when its leftmost visible pixel is at lx.
            // For sprites with x < 0, ready as soon as lx == 0.
            let ready_at = sx.max(0);
            if ready_at == lx as i16 {
                if sx < best_x {
                    best_x = sx;
                    best = Some(i);
                }
            }
        }
        best
    }

    fn tick_bg_fetch(&mut self, bus: &impl PpuBus, line: u8) {
        match self.fetch_step {
            FetchStep::GetTile => {
                self.fetch_sub += 1;
                if self.fetch_sub >= 2 {
                    self.fetch_sub = 0;
                    self.fetch_tile_id = self.read_bg_tile_id(bus, line);
                    self.fetch_step = FetchStep::GetTileLow;
                }
            }
            FetchStep::GetTileLow => {
                self.fetch_sub += 1;
                if self.fetch_sub >= 2 {
                    self.fetch_sub = 0;
                    self.fetch_tile_low = self.read_bg_tile_byte(bus, line, false);
                    self.fetch_step = FetchStep::GetTileHigh;
                }
            }
            FetchStep::GetTileHigh => {
                self.fetch_sub += 1;
                if self.fetch_sub >= 2 {
                    self.fetch_sub = 0;
                    self.fetch_tile_high = self.read_bg_tile_byte(bus, line, true);
                    self.fetch_step = FetchStep::Sleep;
                }
            }
            FetchStep::Sleep => {
                self.fetch_sub += 1;
                if self.fetch_sub >= 2 {
                    self.fetch_sub = 0;
                    self.fetch_step = FetchStep::Push;
                }
            }
            FetchStep::Push => {
                // Push 8 pixels into BG FIFO only if it is empty.
                if self.bg_fifo.is_empty() {
                    for bit in (0..8).rev() {
                        let lo = (self.fetch_tile_low >> bit) & 1;
                        let hi = (self.fetch_tile_high >> bit) & 1;
                        let color = (hi << 1) | lo;
                        self.bg_fifo.push_back(color);
                    }
                    self.fetch_x = self.fetch_x.wrapping_add(1);
                    self.fetch_step = FetchStep::GetTile;
                }
            }
        }
    }

    fn read_bg_tile_id(&self, bus: &impl PpuBus, line: u8) -> u8 {
        let lcdc = bus.lcdc();
        let (map_base, tile_y_in_map, tile_x_in_map) = if self.fetching_window {
            let map = if lcdc.contains(LcdControl::WINDOW_TILE_MAP) {
                0x1C00u16
            } else {
                0x1800u16
            };
            let y = (self.window_line as u16) / 8;
            let x = self.fetch_x as u16;
            (map, y, x)
        } else {
            let map = if lcdc.contains(LcdControl::TILEMAP_AREA) {
                0x1C00u16
            } else {
                0x1800u16
            };
            let scy = bus.scy();
            let scx = bus.scx();
            let y = (line.wrapping_add(scy) as u16) / 8;
            let x = ((self.fetch_x as u16) + (scx as u16) / 8) & 0x1F;
            (map, y, x)
        };
        let addr = map_base + (tile_y_in_map & 0x1F) * 32 + tile_x_in_map;
        bus.read_vram(addr)
    }

    fn read_bg_tile_byte(&self, bus: &impl PpuBus, line: u8, high: bool) -> u8 {
        let lcdc = bus.lcdc();
        let py = if self.fetching_window {
            (self.window_line) % 8
        } else {
            line.wrapping_add(bus.scy()) % 8
        };
        let tile_id = self.fetch_tile_id;
        let tile_addr = if lcdc.contains(LcdControl::TILEDATA_AREA) {
            (tile_id as u16) * 16
        } else {
            // Signed addressing: base $9000 == VRAM offset 0x1000.
            let signed = tile_id as i8 as i16;
            (0x1000i32 + (signed as i32) * 16) as u16
        };
        let addr = tile_addr + (py as u16) * 2 + if high { 1 } else { 0 };
        bus.read_vram(addr)
    }

    fn tick_obj_fetch(&mut self, bus: &impl PpuBus, line: u8) {
        // unwrap safe: caller checks
        let mut f = self.obj_fetch.take().unwrap();
        let sprite = &self.line_sprites[f.sprite_idx];
        let double_height = bus.lcdc().contains(LcdControl::OBJ_SIZE);
        match f.step {
            FetchStep::GetTile => {
                f.sub += 1;
                if f.sub >= 2 {
                    f.sub = 0;
                    f.step = FetchStep::GetTileLow;
                }
            }
            FetchStep::GetTileLow => {
                f.sub += 1;
                if f.sub >= 2 {
                    f.sub = 0;
                    f.tile_low = self.read_obj_byte(bus, sprite, line, double_height, false);
                    f.step = FetchStep::GetTileHigh;
                }
            }
            FetchStep::GetTileHigh => {
                f.sub += 1;
                if f.sub >= 2 {
                    f.sub = 0;
                    f.tile_high = self.read_obj_byte(bus, sprite, line, double_height, true);
                    f.step = FetchStep::Push;
                }
            }
            FetchStep::Sleep => {}
            FetchStep::Push => {
                // Mix-push into OBJ FIFO. Pixels already present are not overwritten
                // unless their color is 0 (transparent). This implements OBJ-vs-OBJ
                // priority (first to arrive wins on non-transparent pixels).
                let skip: u8 = if sprite.x() < 0 {
                    (-sprite.x()) as u8
                } else {
                    0
                };
                for i in 0u8..8u8 {
                    let bit = if sprite.has_x_flip() { i } else { 7 - i };
                    let lo = (f.tile_low >> bit) & 1;
                    let hi = (f.tile_high >> bit) & 1;
                    let color = (hi << 1) | lo;
                    if i < skip {
                        continue;
                    }
                    let slot = (i - skip) as usize;
                    let new_pix = ObjPixel {
                        color,
                        palette_obp1: sprite.palette(),
                        bg_priority: sprite.bg_priority(),
                    };
                    if slot < self.obj_fifo.len() {
                        if self.obj_fifo[slot].color == 0 {
                            self.obj_fifo[slot] = new_pix;
                        }
                    } else {
                        self.obj_fifo.push_back(new_pix);
                    }
                }
                self.obj_fetch = None;
                return;
            }
        }
        self.obj_fetch = Some(f);
    }

    fn read_obj_byte(
        &self,
        bus: &impl PpuBus,
        sprite: &Sprite,
        line: u8,
        double_height: bool,
        high: bool,
    ) -> u8 {
        let mut row = (line as i16 - sprite.y()) as u16;
        let height: u16 = if double_height { 16 } else { 8 };
        if sprite.has_y_flip() {
            row = height - 1 - row;
        }
        let tile_index = if double_height {
            let base = sprite.tile_index() & 0xFE;
            if row < 8 { base } else { base + 1 }
        } else {
            sprite.tile_index()
        };
        let addr = (tile_index as u16) * 16 + (row & 7) * 2 + if high { 1 } else { 0 };
        bus.read_vram(addr)
    }

    fn try_push_pixel(&mut self, bus: &impl PpuBus, line: u8) {
        if self.bg_fifo.is_empty() {
            return;
        }
        let bg_color = self.bg_fifo.pop_front().unwrap();

        // Discard SCX % 8 pixels at start of line (only for BG, not window).
        if self.scx_discard > 0 && !self.fetching_window {
            self.scx_discard -= 1;
            // Also pop the matching OBJ pixel if any.
            self.obj_fifo.pop_front();
            return;
        }

        let bg_enabled = bus.lcdc().contains(LcdControl::BG_WINDOW_ENABLE);
        let lcdc = bus.lcdc();

        // Effective BG color id (0 if BG/Win disabled on DMG).
        let bg_color_id = if bg_enabled { bg_color } else { 0 };

        let obj_pix = self.obj_fifo.pop_front();

        let final_color: u8;
        if let Some(op) = obj_pix {
            if op.color == 0 || !lcdc.contains(LcdControl::OBJ_ENABLE) {
                // OBJ transparent → use BG.
                final_color = if bg_enabled {
                    bus.bgp_color(bg_color_id)
                } else {
                    0
                };
            } else {
                // Non-transparent OBJ: priority handling.
                let obj_wins = if !bg_enabled {
                    true // DMG: BG disabled means OBJ always wins
                } else if op.bg_priority {
                    bg_color_id == 0
                } else {
                    true
                };
                if obj_wins {
                    final_color = if op.palette_obp1 {
                        bus.obp1_color(op.color)
                    } else {
                        bus.obp0_color(op.color)
                    };
                } else {
                    final_color = bus.bgp_color(bg_color_id);
                }
            }
        } else {
            final_color = if bg_enabled {
                bus.bgp_color(bg_color_id)
            } else {
                0
            };
        }

        let idx = (line as usize) * (LCD_WIDTH as usize) + (self.lx as usize);
        if idx < self.frame_buffer.len() {
            self.frame_buffer[idx] = final_color;
        }
        self.lx += 1;
    }
}
