use std::ops::RangeInclusive;

pub(crate) enum Headers {}

impl Headers {
    pub const ROM_TITLE: RangeInclusive<usize> = 0x0134..=0x0143;

    pub const TYPE: usize = 0x0147;
    pub const ROM_SIZE: usize = 0x0148;
    pub const RAM_SIZE: usize = 0x0149;
}
