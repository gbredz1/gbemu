#[cfg(test)]
mod tests {
    use crate::cpu::Flags;
    use crate::cpu::addressing_mode::{Op, Reg};
    use crate::cpu::instruction::Operation::*;
    use crate::cpu::instruction::{Instruction, Operation};
    use crate::tests::bus::TestBus;
    use crate::{Cpu, z};

    macro_rules! f {
        ($z:expr, $n:expr, $h:expr, $c:expr) => {
            FlagsTest {
                z: $z == 1,
                n: $n == 1,
                h: $h == 1,
                c: $c == 1,
            }
        };
    }
    macro_rules! output {
        ($o:expr) => {
            match $o {
                "a" => |m: &TestMachine| m.cpu.a(),
                _ => panic!("Invalid"),
            }
        };
    }

    struct FlagsTest {
        z: bool,
        n: bool,
        h: bool,
        c: bool,
    }

    struct TestMachine {
        cpu: Cpu,
        bus: TestBus,
        instr: Instruction,
    }

    impl TestMachine {
        fn with_operation(operation: Operation) -> Self {
            Self {
                cpu: Cpu::default(),
                bus: TestBus::default(),
                instr: Instruction::from(operation, 0, 0, 0),
            }
        }

        fn clear_flags(&mut self) -> &mut Self {
            self.cpu.clear_flag(Flags::Z | Flags::N | Flags::H | Flags::C);
            self
        }

        fn set_flags(&mut self, flags: Flags) -> &mut Self {
            self.cpu.set_flag(flags);
            self
        }

        fn run(&mut self, data: Vec<u8>) -> &mut Self {
            self.instr.execute(&mut self.cpu, &mut self.bus, &data);
            self
        }

        fn check_flags(&mut self, op1: u8, op2: u8, expected_flags: FlagsTest) {
            self.cpu.set_a(op1);
            self.run(vec![op2]);

            self.assert_flags(Flags::Z, expected_flags.z);
            self.assert_flags(Flags::N, expected_flags.n);
            self.assert_flags(Flags::H, expected_flags.h);
            self.assert_flags(Flags::C, expected_flags.c);
        }

        fn check_result<T: PartialEq + std::fmt::Debug + std::fmt::LowerHex>(
            &mut self,
            op1_val: u8,
            op2_val: u8,
            expected_result: T,
            expected_flags: FlagsTest,
            output: impl FnOnce(&Self) -> T,
        ) {
            self.check_flags(op1_val, op2_val, expected_flags);
            let output_val = output(self);
            assert_eq!(
                output_val, expected_result,
                "Result incorrect : 0x{:x} != 0x{:x}",
                output_val, expected_result
            );
        }

        fn assert_flags(&self, flags: Flags, expected_value: bool) {
            assert_eq!(self.cpu.flag(flags), expected_value, "flag {:?} incorrect", flags);
        }
    }

    #[test]
    fn test_cp() {
        let mut m = TestMachine::with_operation(CP(z!("n")));

        // Test zero flag (A == n)
        m.clear_flags();
        m.check_flags(0x42, 0x42, f!(1, 1, 0, 0));
        // Test non-zero result (A > n)
        m.clear_flags();
        m.check_flags(0b1000_0000, 0b0001_0000, f!(0, 1, 0, 0));
        // Test half carry (A & 0x0F < n & 0x0F)
        m.clear_flags();
        m.check_flags(0b0001_0000, 0b0000_0001, f!(0, 1, 1, 0));
        // Test carry flag (A < n) = 0b1111_1111
        m.clear_flags();
        m.check_flags(0b0000_0001, 0b0000_0010, f!(0, 1, 1, 1));
        // Test carry flag (A < n) = 0b1111_0000
        m.clear_flags();
        m.check_flags(0b0000_0000, 0b0001_0000, f!(0, 1, 0, 1));
    }

    #[test]
    fn test_sub() {
        let mut m = TestMachine::with_operation(SUB(z!("n")));

        // Test zero flag (A == n)
        m.check_result(0x42, 0x42, 0x00, f!(1, 1, 0, 0), |m| m.cpu.a());
        // Test non-zero result (A > n)
        m.check_result(0b1000_0000, 0b0001_0000, 0b0111_0000, f!(0, 1, 0, 0), |m| m.cpu.a());
        // Test half carry (A & 0x0F < n & 0x0F)
        m.check_result(0b0001_0000, 0b0000_0001, 0b0000_1111, f!(0, 1, 1, 0), |m| m.cpu.a());
        // Test carry flag (A < n) = 0b1111_1111
        m.check_result(0b0000_0001, 0b0000_0010, 0b1111_1111, f!(0, 1, 1, 1), |m| m.cpu.a());
        // Test carry flag (A < n) = 0b1111_0000
        m.check_result(0b0000_0000, 0b0001_0000, 0b1111_0000, f!(0, 1, 0, 1), |m| m.cpu.a());
    }

    #[test]
    fn test_add() {
        let mut m = TestMachine::with_operation(ADD(z!("A"), z!("n")));

        // Test zero flag (A == n)
        m.clear_flags();
        m.check_result(0x00, 0x00, 0x00, f!(1, 0, 0, 0), output!("a"));
        // Test non-zero result (A + n < 256)
        m.clear_flags();
        m.check_result(0b1000_0000, 0b0001_0000, 0b1001_0000, f!(0, 0, 0, 0), output!("a"));
        // Test half carry
        m.clear_flags();
        m.check_result(0b0000_1111, 0b0000_0001, 0b0001_0000, f!(0, 0, 1, 0), output!("a"));
        // Test carry flag
        m.clear_flags();
        m.check_result(0b1111_0000, 0b0001_0010, 0b0000_0010, f!(0, 0, 0, 1), output!("a"));
        // Test carry and half flags
        m.clear_flags();
        m.check_result(0b1100_1100, 0b0111_0111, 0b0100_0011, f!(0, 0, 1, 1), output!("a"));
    }

    #[test]
    fn test_scf() {
        let mut m = TestMachine::with_operation(SCF);

        // Case 1: All flags are erased.
        m.clear_flags().run(vec![]);
        m.assert_flags(Flags::Z, false);
        m.assert_flags(Flags::N, false);
        m.assert_flags(Flags::H, false);
        m.assert_flags(Flags::C, true);

        // Case 2: The Z flag is active
        m.clear_flags().set_flags(Flags::Z).run(vec![]);
        m.assert_flags(Flags::Z, true);
        m.assert_flags(Flags::N, false);
        m.assert_flags(Flags::H, false);
        m.assert_flags(Flags::C, true);
    }

    #[test]
    fn test_daa() {
        let mut m = TestMachine::with_operation(DAA);

        // --- Tests after addition (N=0) ---

        // Normal case (without adjustment)
        m.clear_flags();
        m.check_result(0x45, 0x00, 0x45, f!(0, 0, 0, 0), output!("a"));

        // Lower nibble value > 9
        m.clear_flags();
        m.check_result(0x4A, 0x00, 0x50, f!(0, 0, 0, 0), output!("a"));

        // Case with H flag defined
        m.clear_flags().set_flags(Flags::H);
        m.check_result(0x45, 0x00, 0x4b, f!(0, 0, 0, 0), output!("a"));

        // Case with value > 0x99
        m.clear_flags();
        m.check_result(0xA5, 0x00, 0x05, f!(0, 0, 0, 1), output!("a"));

        // Case with C flag set
        m.clear_flags().set_flags(Flags::C);
        m.check_result(0x45, 0x00, 0xA5, f!(0, 0, 0, 1), output!("a"));

        // Special case - zero result
        m.clear_flags();
        m.check_result(0x00, 0x00, 0x00, f!(1, 0, 0, 0), output!("a"));

        // Special case - result 0x9A becoming 0x00 with carry
        m.clear_flags();
        m.check_result(0x9A, 0x00, 0x00, f!(1, 0, 0, 1), output!("a"));

        // --- Tests after subtraction (N=1) ---

        // Normal case (without adjustment)
        m.clear_flags().set_flags(Flags::N);
        m.check_result(0x45, 0x00, 0x45, f!(0, 1, 0, 0), output!("a"));

        // Case with H flag defined
        m.clear_flags().set_flags(Flags::N | Flags::H);
        m.check_result(0x45, 0x00, 0x3F, f!(0, 1, 0, 0), output!("a"));

        // Case with flag C defined
        m.clear_flags().set_flags(Flags::N | Flags::C);
        m.check_result(0x45, 0x00, 0xE5, f!(0, 1, 0, 1), output!("a"));

        // Case with H and C flags set
        m.clear_flags().set_flags(Flags::N | Flags::H | Flags::C);
        m.check_result(0x45, 0x00, 0xDF, f!(0, 1, 0, 1), output!("a"));
    }
}
