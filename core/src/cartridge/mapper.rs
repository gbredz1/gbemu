use crate::cartridge::*;

pub(crate) enum Mapper {
    RomOnly(RomOnly),
}

impl MapperTrait for Mapper {
    fn read_rom(&self, rom: &[u8], address: u16) -> u8 {
        match self {
            Mapper::RomOnly(m) => m.read_rom(rom, address),
        }
    }
    fn write_rom(&mut self, rom: &mut [u8], address: u16, byte: u8) {
        match self {
            Mapper::RomOnly(m) => m.write_rom(rom, address, byte),
        }
    }
}

pub(crate) trait MapperTrait {
    fn read_rom(&self, rom: &[u8], address: u16) -> u8;
    fn write_rom(&mut self, rom: &mut [u8], address: u16, byte: u8);
}
