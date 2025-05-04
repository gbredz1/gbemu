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

    impl Default for TestMachine {
        fn default() -> Self {
            Self {
                cpu: Cpu::default(),
                bus: TestBus::default(),
                instr: Instruction::from(NOP, 0, 0, 0),
            }
        }
    }

    impl TestMachine {
        fn set_operation(&mut self, operation: Operation) -> &mut Self {
            self.instr = Instruction::from(operation, 0, 0, 0);
            self
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
            self.clear_flags();
            self.cpu.set_a(op1);
            self.run(vec![op2]);

            self.assert_flags(Flags::Z, expected_flags.z);
            self.assert_flags(Flags::N, expected_flags.n);
            self.assert_flags(Flags::H, expected_flags.h);
            self.assert_flags(Flags::C, expected_flags.c);
        }

        fn check(
            &mut self,
            op1_val: u8,
            op2_val: u8,
            expected_result: u8,
            expected_flags: FlagsTest,
            result_validator: impl FnOnce(&mut Self, u8, u8, u8),
        ) {
            self.check_flags(op1_val, op2_val, expected_flags);
            result_validator(self, op1_val, op2_val, expected_result);
        }

        fn assert_flags(&self, flags: Flags, expected_value: bool) {
            assert_eq!(self.cpu.flag(flags), expected_value, "flag {:?} incorrect", flags);
        }
    }

    #[test]
    fn test_cp() {
        let mut m = TestMachine::default();
        m.set_operation(CP(z!("n")));

        // Test zero flag (A == n)
        m.check_flags(0x42, 0x42, f!(1, 1, 0, 0));
        // Test non-zero result (A > n)
        m.check_flags(0b1000_0000, 0b0001_0000, f!(0, 1, 0, 0));
        // Test half carry (A & 0x0F < n & 0x0F)
        m.check_flags(0b0001_0000, 0b0000_0001, f!(0, 1, 1, 0));
        // Test carry flag (A < n) = 0b1111_1111
        m.check_flags(0b0000_0001, 0b0000_0010, f!(0, 1, 1, 1));
        // Test carry flag (A < n) = 0b1111_0000
        m.check_flags(0b0000_0000, 0b0001_0000, f!(0, 1, 0, 1));
    }

    #[test]
    fn test_sub() {
        let mut m = TestMachine::default();
        m.set_operation(SUB(z!("n")));

        let extra = |m: &mut TestMachine, _op1, _op2, res| {
            assert_eq!(m.cpu.a(), res, "Result incorrect");
        };

        // Test zero flag (A == n)
        m.check(0x42, 0x42, 0x00, f!(1, 1, 0, 0), extra);
        // Test non-zero result (A > n)
        m.check(0b1000_0000, 0b0001_0000, 0b0111_0000, f!(0, 1, 0, 0), extra);
        // Test half carry (A & 0x0F < n & 0x0F)
        m.check(0b0001_0000, 0b0000_0001, 0b0000_1111, f!(0, 1, 1, 0), extra);
        // Test carry flag (A < n) = 0b1111_1111
        m.check(0b0000_0001, 0b0000_0010, 0b1111_1111, f!(0, 1, 1, 1), extra);
        // Test carry flag (A < n) = 0b1111_0000
        m.check(0b0000_0000, 0b0001_0000, 0b1111_0000, f!(0, 1, 0, 1), extra);
    }

    #[test]
    fn test_add() {
        let mut m = TestMachine::default();
        m.set_operation(ADD(z!("A"), z!("n")));

        let extra = |m: &mut TestMachine, _op1, _op2, res| {
            assert_eq!(m.cpu.a(), res, "Result incorrect");
        };

        // Test zero flag (A == n)
        m.check(0x00, 0x00, 0x00, f!(1, 0, 0, 0), extra);
        // Test non-zero result (A + n < 256)
        m.check(0b1000_0000, 0b0001_0000, 0b1001_0000, f!(0, 0, 0, 0), extra);
        // Test half carry
        m.check(0b0000_1111, 0b0000_0001, 0b0001_0000, f!(0, 0, 1, 0), extra);
        // Test carry flag
        m.check(0b1111_0000, 0b0001_0010, 0b0000_0010, f!(0, 0, 0, 1), extra);
        // Test carry and half flags
        m.check(0b1100_1100, 0b0111_0111, 0b0100_0011, f!(0, 0, 1, 1), extra);
    }

    #[test]
    fn test_scf() {
        let mut m = TestMachine::default();
        m.set_operation(SCF);

        // Cas 1: Tous les drapeaux sont effacés
        m.clear_flags().run(vec![]);
        m.assert_flags(Flags::Z, false);
        m.assert_flags(Flags::N, false);
        m.assert_flags(Flags::H, false);
        m.assert_flags(Flags::C, true);

        // Cas 2: Le drapeau Z est actif
        m.clear_flags().set_flags(Flags::Z).run(vec![]);
        m.assert_flags(Flags::Z, true);
        m.assert_flags(Flags::N, false);
        m.assert_flags(Flags::H, false);
        m.assert_flags(Flags::C, true);
    }
}
