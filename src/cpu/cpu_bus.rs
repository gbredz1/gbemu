pub trait CpuBus {
    fn read_byte(&self, addr: u16) -> u8;
    fn write_byte(&mut self, addr: u16, byte: u8);
    fn read_word(&self, addr: u16) -> u16;
    fn write_word(&mut self, addr: u16, word: u16);
}
