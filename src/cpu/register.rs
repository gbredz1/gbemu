use crate::cpu::CPU;

#[repr(C)] // to keep order
union UnsafeRegister16 {
    value: u16,
    bytes: (u8, u8), // (low, high) => little-endian
}

struct Register16 {
    register: UnsafeRegister16,
}

impl Register16 {
    fn new() -> Self {
        Self {
            register: UnsafeRegister16 { value: 0 },
        }
    }

    fn value(&self) -> u16 {
        unsafe { self.register.value }
    }
    fn set_value(&mut self, value: u16) {
        unsafe { self.register.value = value }
    }
    fn high(&self) -> u8 {
        unsafe { self.register.bytes.1 }
    }
    fn set_high(&mut self, high: u8) {
        unsafe { self.register.bytes.1 = high }
    }
    fn set_low(&mut self, low: u8) {
        unsafe { self.register.bytes.0 = low }
    }
    fn low(&self) -> u8 {
        unsafe { self.register.bytes.0 }
    }
}

