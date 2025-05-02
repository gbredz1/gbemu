#[derive(Debug, Clone, Copy)]
pub(crate) enum Mode {
    HBlank = 0,        // 87-204 cycles
    VBlank = 1,        // 4560 cycles ( 10 lines x 456 cycles)
    OAMScan = 2,       // 80 cycles
    PixelTransfer = 3, // 172-289 cycles
}
