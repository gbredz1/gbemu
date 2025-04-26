mod addressing_mode;
mod cpu_bus;

use crate::bus::Interrupt;
use crate::cpu::addressing_mode::CC;
pub(crate) use crate::cpu::cpu_bus::CpuBus;
use crate::cpu::register::Register16;
use bitflags::bitflags;
use log::trace;

mod decoder;
mod instruction;

#[cfg(test)]
mod decoder_test;
mod display;
mod register;

use crate::cpu_decode;

bitflags! {
    pub struct Flags: u8 {
        const Z = 0b1000_0000;
        const N = 0b0100_0000;
        const H = 0b0010_0000;
        const C = 0b0001_0000;
    }
}

pub struct Cpu {
    af: Register16,
    bc: Register16,
    de: Register16,
    hl: Register16,
    sp: u16,
    pc: u16,
    halted: bool,
    ime: bool,
    ime_scheduled: bool,
}

impl Default for Cpu {
    fn default() -> Self {
        Cpu {
            af: Register16::new(0x01B0), // BMG = $01.., GGC = $11..
            bc: Register16::new(0x0013),
            de: Register16::new(0x00D8),
            hl: Register16::new(0x014D),
            sp: 0xFFFE,
            pc: 0x0100,
            halted: false,
            ime: false,
            ime_scheduled: false,
        }
    }
}

impl Cpu {
    pub fn step(&mut self, bus: &mut impl CpuBus) -> Result<usize, String> {
        let interrupt_cycles = self.handle_interrupt(bus);
        if interrupt_cycles > 0 {
            return Ok(interrupt_cycles);
        }

        if self.halted {
            return Ok(4);
        }

        if self.ime_scheduled {
            self.ime = true;
            self.ime_scheduled = false;
        }

        let opcode_addr = self.pc;
        let opcode = self.pc_read_byte(bus);

        let instruction = cpu_decode!(opcode);
        let instruction = match instruction {
            Some(instruction) => instruction,
            None => {
                return Err(format!("Instruction not found: {:02X}", opcode));
            }
        };

        let mut data = vec![];
        for _ in 1..instruction.size {
            data.push(self.pc_read_byte(bus));
        }

        let opcode_debug = format!(
            "${:04X} > {:02X}{:<20}; {}",
            opcode_addr,
            opcode,
            if !data.is_empty() {
                format!(
                    " {}",
                    data.iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<String>>()
                        .join(" ")
                )
            } else {
                String::new()
            },
            instruction.operation,
        );
        trace!("{}", opcode_debug);

        Ok(instruction.execute(self, bus, data))
    }
    pub fn reset(&mut self) {
        *self = Cpu::default();
    }

    fn pc_read_byte(&mut self, bus: &impl CpuBus) -> u8 {
        let byte = bus.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);

        byte
    }
    fn sp_push_word(&mut self, bus: &mut impl CpuBus, value: u16) {
        // bus.write_byte(self.sp, (value >> 8) as u8);
        // self.sp = self.sp.wrapping_sub(1);
        // bus.write_byte(self.sp, value as u8);
        // self.sp = self.sp.wrapping_sub(1);

        self.sp = self.sp.wrapping_sub(2);
        bus.write_word(self.sp, value);
    }
    fn sp_pop_word(&mut self, bus: &mut impl CpuBus) -> u16 {
        // let high = bus.read_byte(self.sp) as u16;
        // self.sp = self.sp.wrapping_add(1);
        // let low = bus.read_byte(self.sp) as u16;
        // self.sp = self.sp.wrapping_add(1);
        //
        // high << 8 | low

        let value = bus.read_word(self.sp);
        self.sp = self.sp.wrapping_add(2);
        value
    }

    fn handle_interrupt(&mut self, bus: &mut impl CpuBus) -> usize {
        if self.halted {
            let if_val = bus.interrupt_flag();
            let ie_val = bus.interrupt_enable();

            if !(if_val & ie_val).is_empty() {
                self.halted = false;
            }

            if !self.ime {
                return 0;
            }
        } else if !self.ime {
            return 0;
        }

        let if_val = bus.interrupt_flag();
        let ie_val = bus.interrupt_enable();
        let triggered = if_val & ie_val;
        if triggered.is_empty() {
            return 0;
        }

        // Disable IME
        self.ime = false;
        self.ime_scheduled = false;

        // Save current PC on the stack
        self.sp = self.sp.wrapping_sub(2);
        bus.write_word(self.sp, self.pc);

        // Determine which interrupt to handle (priority: VBlank > LCD STAT > Timer > Serial > Joypad)
        let interrupt_vector = if triggered.contains(Interrupt::VBLANK) {
            // Clear VBlank interrupt flag
            bus.update_interrupt_flag(Interrupt::VBLANK, false);
            0x0040 // VBlank interrupt address
        } else if triggered.contains(Interrupt::LCD_STAT) {
            bus.update_interrupt_flag(Interrupt::LCD_STAT, false);
            0x0048 // LCD STAT interrupt address
        } else if triggered.contains(Interrupt::TIMER) {
            bus.update_interrupt_flag(Interrupt::TIMER, false);
            0x0050 // Timer interrupt address
        } else if triggered.contains(Interrupt::SERIAL) {
            bus.update_interrupt_flag(Interrupt::SERIAL, false);
            0x0058 // Serial interrupt address
        } else if triggered.contains(Interrupt::JOYPAD) {
            bus.update_interrupt_flag(Interrupt::JOYPAD, false);
            0x0060 // Joypad interrupt address
        } else {
            unreachable!("No interrupts triggered despite previous checks");
        };

        // Set PC to interrupt address
        self.pc = interrupt_vector;

        // Processing an interrupt takes 20 cycles
        20
    }

    // Registers accessors 8 bits
    pub fn a(&self) -> u8 {
        self.af.high()
    }
    pub fn set_a(&mut self, value: u8) {
        self.af.set_high(value);
    }
    pub fn b(&self) -> u8 {
        self.bc.high()
    }
    pub fn set_b(&mut self, value: u8) {
        self.bc.set_high(value);
    }
    pub fn c(&self) -> u8 {
        self.bc.low()
    }
    pub fn set_c(&mut self, value: u8) {
        self.bc.set_low(value);
    }
    pub fn d(&self) -> u8 {
        self.de.high()
    }
    pub fn set_d(&mut self, value: u8) {
        self.de.set_high(value);
    }
    pub fn e(&self) -> u8 {
        self.de.low()
    }
    pub fn set_e(&mut self, value: u8) {
        self.de.set_low(value);
    }
    pub fn f(&self) -> u8 {
        self.af.low()
    }
    pub fn set_f(&mut self, value: u8) {
        self.af.set_low(value);
    }
    pub fn h(&self) -> u8 {
        self.hl.high()
    }
    pub fn set_h(&mut self, value: u8) {
        self.hl.set_high(value);
    }
    pub fn l(&self) -> u8 {
        self.hl.low()
    }
    pub fn set_l(&mut self, value: u8) {
        self.hl.set_low(value);
    }

    // Register 16-bits accessors
    pub fn af(&self) -> u16 {
        self.af.value()
    }
    pub fn set_af(&mut self, value: u16) {
        self.af.set_value(value)
    }
    pub fn bc(&self) -> u16 {
        self.bc.value()
    }
    pub fn set_bc(&mut self, value: u16) {
        self.bc.set_value(value)
    }
    pub fn de(&self) -> u16 {
        self.de.value()
    }
    pub fn set_de(&mut self, value: u16) {
        self.de.set_value(value)
    }
    pub fn hl(&self) -> u16 {
        self.hl.value()
    }
    pub fn set_hl(&mut self, value: u16) {
        self.hl.set_value(value)
    }
    pub fn sp(&self) -> u16 {
        self.sp
    }
    pub fn set_sp(&mut self, value: u16) {
        self.sp = value
    }
    pub fn pc(&self) -> u16 {
        self.pc
    }
    pub fn set_pc(&mut self, value: u16) {
        self.pc = value
    }

    // Flags accessors
    pub fn flag(&self, flag: Flags) -> bool {
        Flags::from_bits_truncate(self.f()).contains(flag)
    }
    pub fn set_flag(&mut self, flag: Flags) {
        self.set_f(self.f() | flag.bits());
    }
    pub fn clear_flag(&mut self, flag: Flags) {
        self.set_f(self.f() & !flag.bits());
    }
    pub fn set_flag_if(&mut self, flag: Flags, condition: bool) {
        if condition {
            self.set_flag(flag);
        } else {
            self.clear_flag(flag);
        }
    }
    pub fn check_condition(&self, condition: CC) -> bool {
        match condition {
            CC::NZ => !self.flag(Flags::Z),
            CC::Z => self.flag(Flags::Z),
            CC::NC => !self.flag(Flags::C),
            CC::C => self.flag(Flags::C),
        }
    }

    // Internal methods
    pub fn ime(&self) -> bool {
        self.ime
    }
    pub fn set_ime(&mut self, value: bool) {
        self.ime = value;
    }
    pub fn halt(&self) -> bool {
        self.halted
    }
    pub fn set_halted(&mut self, value: bool) {
        self.halted = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::BusIO;
    use crate::tests::bus::TestBus;

    impl CpuBus for TestBus {}

    #[test]
    fn test_flags() {
        let mut cpu = Cpu::default();
        cpu.set_f(0x00); // clear all flags

        // Test that flags are cleared
        assert!(!cpu.flag(Flags::Z));
        assert!(!cpu.flag(Flags::N));
        assert!(!cpu.flag(Flags::H));
        assert!(!cpu.flag(Flags::C));

        // Test setting a single flag
        cpu.set_flag(Flags::Z);
        assert!(cpu.flag(Flags::Z));
        assert!(!cpu.flag(Flags::N));

        // Test clearing a single flag
        cpu.clear_flag(Flags::Z);
        assert!(!cpu.flag(Flags::Z));

        // Test setting multiple flags
        cpu.set_flag(Flags::N);
        cpu.set_flag(Flags::H);
        assert!(cpu.flag(Flags::N));
        assert!(cpu.flag(Flags::H));
        assert!(!cpu.flag(Flags::C));

        // Test `set_flag_if` method
        cpu.set_flag_if(Flags::C, true);
        assert!(cpu.flag(Flags::C));
        cpu.set_flag_if(Flags::C, false);
        assert!(!cpu.flag(Flags::C));

        // Verify no unintended flag manipulation
        assert!(cpu.flag(Flags::N));
        assert!(cpu.flag(Flags::H));
        assert!(!cpu.flag(Flags::Z));

        // Test `check_condition` method
        assert!(cpu.check_condition(CC::NZ)); // Not Zero flag should be true
        cpu.set_flag(Flags::Z);
        assert!(cpu.check_condition(CC::Z)); // Zero flag should now be true
        assert!(!cpu.check_condition(CC::NZ)); // Not Zero flag should now be false
        cpu.clear_flag(Flags::Z);

        cpu.set_flag(Flags::C);
        assert!(cpu.check_condition(CC::C)); // Carry flag should be true
        assert!(!cpu.check_condition(CC::NC)); // Not Carry should be false
    }

    #[test]
    fn test_stack_operations() {
        let mut cpu = Cpu::default();
        let mut bus = TestBus::default();
        cpu.set_sp(0xCFFF);

        let initial_value = 0x1234;

        // push
        cpu.sp_push_word(&mut bus, initial_value);
        assert_eq!(cpu.sp(), 0xCFFD, "Stack pointer should be decremented by 2");
        assert_eq!(bus.read_word(cpu.sp()), initial_value, "Stack value should be written");

        // value written at 0xCFFD and 0xCFFE
        let high = bus.read_byte(0xCFFE);
        let low = bus.read_byte(0xCFFD);
        assert_eq!(low, initial_value as u8, "Low byte should be written first");
        assert_eq!(high, (initial_value >> 8) as u8, "High byte should be written second");

        // pop
        let actual_value = cpu.sp_pop_word(&mut bus);
        assert_eq!(cpu.sp(), 0xCFFF, "Stack pointer should be incremented by 2");
        assert_eq!(actual_value, initial_value, "Stack value should be read");
    }
}
