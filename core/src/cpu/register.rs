#[repr(C)] // to keep order
union UnsafeRegister16 {
    value: u16,
    bytes: (u8, u8), // (low, high) => little-endian
}

pub(crate) struct Register16 {
    register: UnsafeRegister16,
}

impl Register16 {
    pub fn new(value: u16) -> Self {
        Self {
            register: UnsafeRegister16 { value },
        }
    }

    pub fn value(&self) -> u16 {
        unsafe { self.register.value }
    }
    pub fn set_value(&mut self, value: u16) {
        self.register.value = value;
    }
    pub fn high(&self) -> u8 {
        unsafe { self.register.bytes.1 }
    }
    #[allow(unused_unsafe)] // ## E0133 mismatch
    pub fn set_high(&mut self, high: u8) {
        unsafe {
            self.register.bytes.1 = high;
        }
    }
    #[allow(unused_unsafe)] // ## E0133 mismatch
    pub fn set_low(&mut self, low: u8) {
        unsafe {
            self.register.bytes.0 = low;
        }
    }
    pub fn low(&self) -> u8 {
        unsafe { self.register.bytes.0 }
    }
}
