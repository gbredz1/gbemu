use crate::cartridge::*;

pub(crate) enum Mapper {
    RomOnly(RomOnly),
    Mbc1(Mbc1),
}

impl MapperTrait for Mapper {
    fn read(&self, rom: &[u8], ram: Option<&[u8]>, address: u16) -> u8 {
        match self {
            Mapper::RomOnly(m) => m.read(rom, ram, address),
            Mapper::Mbc1(m) => m.read(rom, ram, address),
        }
    }
    fn write(&mut self, rom: &[u8], ram: Option<&mut [u8]>, address: u16, byte: u8) {
        match self {
            Mapper::RomOnly(m) => m.write(rom, ram, address, byte),
            Mapper::Mbc1(m) => m.write(rom, ram, address, byte),
        }
    }
}

pub(crate) trait MapperTrait {
    fn read(&self, rom: &[u8], ram: Option<&[u8]>, address: u16) -> u8;
    fn write(&mut self, rom: &[u8], ram: Option<&mut [u8]>, address: u16, byte: u8);
}
