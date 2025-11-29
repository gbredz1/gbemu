use super::mapper::MapperTrait;

pub struct RomOnly;
impl MapperTrait for RomOnly {
    fn read_rom(&self, rom: &[u8], address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => unsafe { *rom.get_unchecked(address as usize) }, // 32KB ROM
            _ => 0xFF,
        }
    }

    fn write_rom(&mut self, _rom: &mut [u8], _address: u16, _byte: u8) {
        // ROM-only cartridges ignore writes
    }
}
