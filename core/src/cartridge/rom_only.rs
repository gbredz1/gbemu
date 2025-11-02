use super::mapper::MapperTrait;

pub struct RomOnly;
impl MapperTrait for RomOnly {
    fn read_rom(&self, rom: &[u8], address: u16) -> u8 {
        unsafe { *rom.get_unchecked(address as usize) }
    }

    fn write_rom(&mut self, rom: &mut [u8], address: u16, byte: u8) {
        unsafe {
            *rom.get_unchecked_mut(address as usize) = byte;
        }
    }
}
