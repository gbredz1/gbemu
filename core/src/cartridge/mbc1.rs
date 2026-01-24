use super::mapper::MapperTrait;
use crate::cartridge::{RAM_BANK_SIZE, ROM_BANK_SIZE};

#[derive(Default)]
pub struct Mbc1 {
    rom_bank: usize,
    mode_ram_banking: bool,
    ram_enabled: bool,
    ram_bank: usize,
    rom_bank_count: usize,
    ram_bank_count: usize,
}

impl Mbc1 {
    pub(crate) fn new(rom_bank_count: usize, ram_bank_count: usize) -> Self {
        Self {
            rom_bank: 1,
            rom_bank_count,
            ram_bank_count,
            ..Self::default()
        }
    }

    #[inline(always)]
    fn current_rom_bank_0000(&self) -> usize {
        if self.mode_ram_banking {
            self.rom_bank & 0b0110_0000
        } else {
            0
        }
    }

    /// map low5=0 to 1
    /// modulo total bank count
    #[inline(always)]
    fn current_rom_bank_4000(&self) -> usize {
        let low5 = self.rom_bank & 0b0001_1111;
        let low5_nonzero = low5 | (low5 == 0) as usize; // map 0 -> 1
        ((self.rom_bank & 0b0110_0000) | low5_nonzero) % self.rom_bank_count
    }

    #[inline(always)]
    fn read_handler_noop(&self, _: &[u8], _: Option<&[u8]>, _: u16) -> u8 {
        0xFF
    }
    #[inline(always)]
    fn write_handler_noop(_: &mut Mbc1, _: Option<&mut [u8]>, _: u16, _: u8) {}

    /// read $0000..$3FFF
    #[inline(always)]
    fn read_handler_rom_0000(&self, rom: &[u8], _: Option<&[u8]>, address: u16) -> u8 {
        let bank = self.current_rom_bank_0000() % self.rom_bank_count;
        let idx = bank * ROM_BANK_SIZE + (address as usize);

        unsafe { *rom.get_unchecked(idx) }
    }
    /// read $4000..$7FFF : rom
    #[inline(always)]
    fn read_handler_rom_4000(&self, rom: &[u8], _: Option<&[u8]>, address: u16) -> u8 {
        let bank = self.current_rom_bank_4000() % self.rom_bank_count;
        let idx = bank * ROM_BANK_SIZE + (address as usize - ROM_BANK_SIZE);

        unsafe { *rom.get_unchecked(idx) }
    }

    /// read $A000..$BFFF : ram
    #[inline(always)]
    fn read_handler_ram(&self, _: &[u8], ram: Option<&[u8]>, address: u16) -> u8 {
        let Some(ram) = ram else { return 0xFF };
        if !self.ram_enabled || self.ram_bank_count == 0 {
            return 0xFF;
        }

        let bank = if self.mode_ram_banking {
            self.ram_bank % self.ram_bank_count
        } else {
            0
        };

        let idx = bank * RAM_BANK_SIZE + ((address as usize - 0xA000) & (RAM_BANK_SIZE - 1));

        unsafe { *ram.get_unchecked(idx) }
    }

    /// write $0000..$1FFF: $A => ram=on else ram=off
    #[inline(always)]
    fn write_handler_set_ram_state(&mut self, _: Option<&mut [u8]>, _: u16, byte: u8) {
        self.ram_enabled = byte & 0x0F == 0x0A;
    }

    /// write $2000..$3FFF: set ROM bank (5bits)
    #[inline(always)]
    fn write_handler_set_rom_bank(&mut self, _: Option<&mut [u8]>, _: u16, byte: u8) {
        let low5 = (byte & 0x1F) as usize;
        // let low5 = if low5 == 0 { 1 } else { low5 };
        self.rom_bank = (self.rom_bank & 0b1110_0000) | low5; // set low 5 bits
    }

    /// write $4000..$5FFF: set RAM bank (2bits)
    #[inline(always)]
    fn write_handler_set_ram_bank(&mut self, _: Option<&mut [u8]>, _: u16, byte: u8) {
        let bits = (byte & 0b0000_00011) as usize;

        // the 2-bit register is always written
        self.rom_bank = (self.rom_bank & 0b0001_1111) | (bits << 5);

        if self.mode_ram_banking {
            self.ram_bank = bits;
        }
    }

    /// write $6000..$7FFF: set bank mode (1bits)
    #[inline(always)]
    fn write_handler_set_bank_mode(&mut self, _: Option<&mut [u8]>, _: u16, byte: u8) {
        self.mode_ram_banking = (byte & 0x01) != 0;
    }

    /// write $A000..$BFFF: write ram
    #[inline(always)]
    fn write_handler_ram(&mut self, ram: Option<&mut [u8]>, address: u16, byte: u8) {
        let Some(ram) = ram else { return };
        if self.ram_bank_count == 0 || !self.ram_enabled {
            return;
        }

        let bank = if self.mode_ram_banking {
            self.ram_bank % self.ram_bank_count
        } else {
            0
        };

        let idx = (bank << 13) | ((address & 0x1FFF) as usize);
        unsafe {
            *ram.get_unchecked_mut(idx) = byte;
        }
    }
}

type Mbc1WriteHandler = fn(&mut Mbc1, Option<&mut [u8]>, u16, u8);
const WRITE_HANDLERS: [Mbc1WriteHandler; 16] = [
    Mbc1::write_handler_set_ram_state, // $0... ┬─▶ 0000–1FFF — RAM Enable
    Mbc1::write_handler_set_ram_state, // $1... ┘
    Mbc1::write_handler_set_rom_bank,  // $2... ┬─▶ 2000–3FFF — ROM Bank Number
    Mbc1::write_handler_set_rom_bank,  // $3... ┘
    Mbc1::write_handler_set_ram_bank,  // $4... ┬─▶ 4000–5FFF — RAM Bank Number — or — Upper Bits of ROM Bank Number
    Mbc1::write_handler_set_ram_bank,  // $5... ┘
    Mbc1::write_handler_set_bank_mode, // $6... ┬─▶ 6000–7FFF — Banking Mode Select
    Mbc1::write_handler_set_bank_mode, // $7... ┘
    Mbc1::write_handler_noop,          // $8... x
    Mbc1::write_handler_noop,          // $8... x
    Mbc1::write_handler_ram,           // $A... ┬─▶ A000–BFFF - RAM write
    Mbc1::write_handler_ram,           // $B... ┘
    Mbc1::write_handler_noop,          // $C... x
    Mbc1::write_handler_noop,          // $D... x
    Mbc1::write_handler_noop,          // $E... x
    Mbc1::write_handler_noop,          // $F... x
];

type Mbc1ReadHandler = fn(&Mbc1, &[u8], Option<&[u8]>, u16) -> u8;
const READ_HANDLERS: [Mbc1ReadHandler; 16] = [
    Mbc1::read_handler_rom_0000, // $0... ┬─▶ 0000–3FFF — ROM Bank X0
    Mbc1::read_handler_rom_0000, // $1... │
    Mbc1::read_handler_rom_0000, // $2... │
    Mbc1::read_handler_rom_0000, // $3... ┘
    Mbc1::read_handler_rom_4000, // $4... ┬─▶ 4000–7FFF — ROM Bank 01-7F
    Mbc1::read_handler_rom_4000, // $5... │
    Mbc1::read_handler_rom_4000, // $6... │
    Mbc1::read_handler_rom_4000, // $7... ┘
    Mbc1::read_handler_noop,     // $8... x
    Mbc1::read_handler_noop,     // $9... x
    Mbc1::read_handler_ram,      // $A... ┬─▶ A000–BFFF — RAM Bank 00–03,
    Mbc1::read_handler_ram,      // $B... ┘
    Mbc1::read_handler_noop,     // $C... x
    Mbc1::read_handler_noop,     // $D... x
    Mbc1::read_handler_noop,     // $E... x
    Mbc1::read_handler_noop,     // $F... x
];

impl MapperTrait for Mbc1 {
    fn read(&self, rom: &[u8], ram: Option<&[u8]>, address: u16) -> u8 {
        READ_HANDLERS[address as usize >> 12](self, rom, ram, address)
    }

    fn write(&mut self, _rom: &[u8], ram: Option<&mut [u8]>, address: u16, byte: u8) {
        WRITE_HANDLERS[address as usize >> 12](self, ram, address, byte);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const W_ROM_N: u16 = 0x2000;
    const W_RAM_N_OR_HIGH2: u16 = 0x4000;
    const W_BANKING_MODE: u16 = 0x6000;
    const W_RAM_ENABLE: u16 = 0x0000;

    const R_BANK_0: u16 = 0x0100;
    const R_BANK_N: u16 = 0x4000;
    const ADDR_RAM: u16 = 0xA000;

    // Build a ROM where each 16KiB bank is filled with its bank index (0..=0xFF)
    fn build_rom(banks: usize) -> Vec<u8> {
        (0..banks)
            .flat_map(|i| std::iter::repeat_n(i as u8, ROM_BANK_SIZE))
            .collect()
    }

    // Build a RAM where each 8KiB bank is filled with its bank index (0..=0xFF)
    fn build_ram(banks: usize) -> Vec<u8> {
        (0..banks)
            .flat_map(|i| std::iter::repeat_n(i as u8, RAM_BANK_SIZE))
            .collect()
    }

    fn init(rom_banks: usize, ram_banks: usize) -> (Mbc1, Vec<u8>, Option<Vec<u8>>) {
        let rom = build_rom(rom_banks);
        let mbc = Mbc1::new(rom_banks, ram_banks);
        let ram = (ram_banks > 0).then(|| build_ram(ram_banks));

        (mbc, rom, ram)
    }

    // Mode 0: 0000..3FFF = bank 0; 4000..7FFF uses (high2<<5)|low5 with low5=0 => 1
    #[test]
    fn rom_mode0_low5_zero_and_combination() {
        let (mut mbc, rom, _) = init(128, 0);

        // low5=0 maps to 1 while preserving high2
        for n in 0..=3u8 {
            // set RAM Bank to n
            mbc.write(&rom, None, W_RAM_N_OR_HIGH2, n);
            // set ROM Bank n to 0
            mbc.write(&rom, None, W_ROM_N, 0);

            let expected = ((n as usize) << 5) | 1;
            assert_eq!(mbc.read(&rom, None, R_BANK_N), expected as u8); // value = selected bank
        }
    }

    // Mode 1: 0000..3FFF = (high2<<5); 4000..7FFF combines (high2<<5)|low5_nonzero
    #[test]
    fn rom_mode1_split_banks() {
        let (mut mbc, rom, _) = init(128, 0);

        // Enter mode 1 (RAM banking mode)
        mbc.write(&rom, None, W_BANKING_MODE, 1);

        // set ROM Bank upper (mode 1)
        mbc.write(&rom, None, W_RAM_N_OR_HIGH2, 1);
        assert_eq!(mbc.read(&rom, None, R_BANK_0), 0x20); // bank(32)

        // 4000 area: if low5=0 then n=1, combined with high2=1 => (1<<5)|1 = $20 + 1
        mbc.write(&rom, None, W_ROM_N, 0b00);
        assert_eq!(mbc.read(&rom, None, R_BANK_N), 0x21); // bank(33)

        // Change high2 to 3; $4000 = (3<<5)|1 = $32
        mbc.write(&rom, None, W_RAM_N_OR_HIGH2, 0b11);
        assert_eq!(mbc.read(&rom, None, R_BANK_0), 0x60); // bank(96)
        assert_eq!(mbc.read(&rom, None, R_BANK_N), 0x61); // bank(97)

        // Set low5 to 17 ; $4000 = (3<<5)|17 = 113
        mbc.write(&rom, None, W_ROM_N, 17);
        assert_eq!(mbc.read(&rom, None, R_BANK_N), 113); // bank(96 + 17)
    }

    // Large sizes: 8Mbit (64 banks) and 16Mbit (128 banks)
    #[test]
    fn large_rom_sizes_wrap() {
        // 8Mb = 64 banks
        let (mut mbc, rom, _) = init(64, 0);

        mbc.write(&rom, None, W_RAM_N_OR_HIGH2, 3); // high2=3 (96)
        mbc.write(&rom, None, W_ROM_N, 31); // low5=31
        // 96+31 % 64 = 127 % 64 = 63
        assert_eq!(mbc.read(&rom, None, R_BANK_N), 63);

        // 96+1 % 64 = 97 % 64 = 33
        mbc.write(&rom, None, W_ROM_N, 0); // low5=0
        assert_eq!(mbc.read(&rom, None, R_BANK_N), 33);

        // 16Mb = 128 banks
        let (mut mbc, rom, _) = init(128, 0);
        mbc.write(&rom, None, W_RAM_N_OR_HIGH2, 2); // high2=2 (64)
        mbc.write(&rom, None, W_ROM_N, 31); // low5=31

        // 64+95 % 128 = 95
        assert_eq!(mbc.read(&rom, None, R_BANK_N), 95);
        mbc.write(&rom, None, W_ROM_N, 0); // low5=0 => 1
        // 64+1 % 128 = 65
        assert_eq!(mbc.read(&rom, None, R_BANK_N), 65);

        mbc.write(&rom, None, W_RAM_N_OR_HIGH2, 3); // high2=2 (96)
        mbc.write(&rom, None, W_ROM_N, 31); // low5=31
        // 96+31 % 128 = 127
        assert_eq!(mbc.read(&rom, None, R_BANK_N), 127);
    }

    // RAM enable/disable and banking behaviour
    #[test]
    fn ram_enable_disable_and_banking() {
        let (mut mbc, rom, mut ram) = init(64, 4);

        // RAM disabled: read 0xFF, write ignored
        assert_eq!(mbc.read(&rom, ram.as_deref(), ADDR_RAM), 0xFF);
        mbc.write(&rom, ram.as_deref_mut(), ADDR_RAM, 0x12);
        assert_eq!(mbc.read(&rom, ram.as_deref(), ADDR_RAM), 0xFF);

        // Only 0x0A (low nibble) enables
        mbc.write(&rom, ram.as_deref_mut(), W_RAM_ENABLE, 0x0B);
        assert_eq!(mbc.read(&rom, ram.as_deref(), 0xA000), 0xFF);
        mbc.write(&rom, ram.as_deref_mut(), W_RAM_ENABLE, 0x0A);
        assert_eq!(mbc.read(&rom, ram.as_deref(), 0xA000), 0x00);

        // Mode 0: RAM bank is 0 => no change
        mbc.write(&rom, ram.as_deref_mut(), W_RAM_N_OR_HIGH2, 3);
        assert_eq!(mbc.read(&rom, ram.as_deref(), ADDR_RAM), 0);
        mbc.write(&rom, ram.as_deref_mut(), ADDR_RAM + 0x100, 0xAA);

        // Mode 1: RAM bank
        mbc.write(&rom, ram.as_deref_mut(), W_BANKING_MODE, 1);
        mbc.write(&rom, ram.as_deref_mut(), W_RAM_N_OR_HIGH2, 1);
        assert_eq!(mbc.read(&rom, ram.as_deref(), ADDR_RAM), 1);
        mbc.write(&rom, ram.as_deref_mut(), W_RAM_N_OR_HIGH2, 2);
        assert_eq!(mbc.read(&rom, ram.as_deref(), ADDR_RAM), 2);
        mbc.write(&rom, ram.as_deref_mut(), W_RAM_N_OR_HIGH2, 3);
        assert_eq!(mbc.read(&rom, ram.as_deref(), ADDR_RAM), 3);

        // Bank 0 still has previous value
        assert_eq!(ram.as_deref().unwrap()[0x100], 0xAA);

        mbc.write(&rom, None, W_RAM_N_OR_HIGH2, 0);
        assert_eq!(mbc.read(&rom, ram.as_deref(), ADDR_RAM), 0);
        assert_eq!(mbc.read(&rom, ram.as_deref(), ADDR_RAM + 0x100), 0xAA);
        mbc.write(&rom, None, W_RAM_N_OR_HIGH2, 2);

        mbc.write(&rom, None, W_BANKING_MODE, 0);
        assert_eq!(mbc.read(&rom, ram.as_deref(), ADDR_RAM), 0);
        assert_eq!(mbc.read(&rom, ram.as_deref(), ADDR_RAM + 0x100), 0xAA);
    }

    #[test]
    fn odd_sizes_behaviour() {
        let (mut mbc, rom, mut ram) = init(7, 3);

        // Odd ROM banks: check wrapping still applies
        assert_eq!(mbc.read(&rom, ram.as_deref(), R_BANK_0), 0);
        mbc.write(&rom, ram.as_deref_mut(), W_ROM_N, 0); // low5=0 => 1
        assert_eq!(mbc.read(&rom, ram.as_deref(), R_BANK_N), 1);
        mbc.write(&rom, ram.as_deref_mut(), W_RAM_N_OR_HIGH2, 3);
        mbc.write(&rom, ram.as_deref_mut(), W_ROM_N, 31);
        // (3<<5)|31 = 127 -> 127 % 7 = 1
        assert_eq!(mbc.read(&rom, ram.as_deref(), R_BANK_N), (127 % 7) as u8);

        // Odd RAM banks
        mbc.write(&rom, ram.as_deref_mut(), W_RAM_ENABLE, 0x0A);
        mbc.write(&rom, ram.as_deref_mut(), W_BANKING_MODE, 1);

        mbc.write(&rom, ram.as_deref_mut(), W_RAM_N_OR_HIGH2, 2);
        assert_eq!(mbc.read(&rom, ram.as_deref(), ADDR_RAM), 2); // bank 2

        // write + read
        mbc.write(&rom, ram.as_deref_mut(), ADDR_RAM + 0x100, 0x22);
        assert_eq!(mbc.read(&rom, ram.as_deref(), ADDR_RAM + 0x100), 0x22);

        mbc.write(&rom, ram.as_deref_mut(), W_RAM_N_OR_HIGH2, 3);
        assert_eq!(mbc.read(&rom, ram.as_deref(), ADDR_RAM), 0); // % 2 => bank(0..2)
    }
}
