use super::mapper::MapperTrait;

pub struct RomOnly;
impl MapperTrait for RomOnly {
    fn read(&self, rom: &[u8], _: Option<&[u8]>, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => unsafe { *rom.get_unchecked(address as usize) }, // 32KB ROM
            _ => 0xFF,
        }
    }

    fn write(&mut self, _: &[u8], _: Option<&mut [u8]>, _: u16, _: u8) {
        // ROM-only cartridges ignore writes
    }
}
