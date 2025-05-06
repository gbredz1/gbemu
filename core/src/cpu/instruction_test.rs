#[cfg(test)]
mod tests {
    use crate::cpu::Flags;
    use crate::cpu::addressing_mode::{Op, Reg};
    use crate::cpu::instruction::Operation::*;
    use crate::cpu::instruction::{Instruction, Operation};
    use crate::cpu::instruction_test::tests::Value::*;
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
    macro_rules! out8 {
        ($o:expr) => {
            match $o {
                "a" => |m: &TestMachine| m.cpu.a(),
                "b" => |m: &TestMachine| m.cpu.b(),
                _ => panic!("Invalid"),
            }
        };
    }
    macro_rules! out16 {
        ($o:expr) => {
            match $o {
                "sp" => |m: &TestMachine| m.cpu.sp(),
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
        data: Vec<u8>,
        cb_instr: bool,
    }

    enum Value {
        A(u8),
        B(u8),
        SP(u16),
        HL(u16),
    }

    impl TestMachine {
        fn with_operation(operation: Operation) -> Self {
            Self {
                cpu: Cpu::default(),
                bus: TestBus::default(),
                instr: Instruction::from(operation, 0, 0, 0),
                data: vec![],
                cb_instr: false,
            }
        }
        fn with_operation_cb(operation: Operation) -> Self {
            Self {
                cpu: Cpu::default(),
                bus: TestBus::default(),
                instr: Instruction::from(operation, 0, 0, 0),
                data: vec![],
                cb_instr: true,
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

        fn set_data(&mut self, data: &[u8]) -> &mut Self {
            self.data = Vec::from(data);
            self
        }

        fn set(&mut self, value: Value) -> &mut Self {
            match value {
                A(val) => self.cpu.set_a(val),
                B(val) => self.cpu.set_b(val),
                SP(val) => self.cpu.set_sp(val),
                HL(val) => self.cpu.set_hl(val),
            };
            self
        }

        fn run(&mut self) -> &mut Self {
            if self.cb_instr {
                self.instr.execute_cb(&mut self.cpu, &mut self.bus, &self.data);
            } else {
                self.instr.execute(&mut self.cpu, &mut self.bus, &self.data);
            }
            self.data = vec![]; // Reset data

            self
        }

        fn assert_flags(&mut self, expected_flags: FlagsTest) -> &mut Self {
            assert_eq!(self.cpu.flag(Flags::Z), expected_flags.z, "Z flag incorrect");
            assert_eq!(self.cpu.flag(Flags::N), expected_flags.n, "N flag incorrect");
            assert_eq!(self.cpu.flag(Flags::H), expected_flags.h, "H flag incorrect");
            assert_eq!(self.cpu.flag(Flags::C), expected_flags.c, "C flag incorrect");
            self
        }

        fn check_flags(&mut self, expected_flags: FlagsTest) {
            self.run().assert_flags(expected_flags);
        }

        fn check_result<T: PartialEq + std::fmt::Debug + std::fmt::LowerHex>(
            &mut self,
            expected_result: T,
            expected_flags: FlagsTest,
            output: impl FnOnce(&Self) -> T,
        ) {
            self.run().assert_flags(expected_flags);
            let output_val = output(self);
            assert_eq!(
                output_val, expected_result,
                "Result incorrect : 0x{:x} != 0x{:x}",
                output_val, expected_result
            );
        }
    }

    #[test]
    fn test_cp() {
        let mut m = TestMachine::with_operation(CP(z!("n")));

        // Test zero flag (A == n)
        m.clear_flags()
            .set(A(0x42))
            .set_data(&[0x42])
            .check_flags(f!(1, 1, 0, 0));
        // Test non-zero result (A > n)
        m.clear_flags()
            .set(A(0b1000_0000))
            .set_data(&[0b0001_0000])
            .check_flags(f!(0, 1, 0, 0));
        // Test half carry (A & 0x0F < n & 0x0F)
        m.clear_flags()
            .set(A(0b0001_0000))
            .set_data(&[0b0000_0001])
            .check_flags(f!(0, 1, 1, 0));
        // Test carry flag (A < n) = 0b1111_1111
        m.clear_flags()
            .set(A(0b0000_0001))
            .set_data(&[0b0000_0010])
            .check_flags(f!(0, 1, 1, 1));
        // Test carry flag (A < n) = 0b1111_0000
        m.clear_flags()
            .set(A(0b0000_0000))
            .set_data(&[0b0001_0000])
            .check_flags(f!(0, 1, 0, 1));
    }

    #[test]
    fn test_sub() {
        let mut m = TestMachine::with_operation(SUB(z!("n")));

        // Test zero flag (A == n)
        m.clear_flags()
            .set(A(0x42))
            .set_data(&[0x42])
            .check_result(0x00, f!(1, 1, 0, 0), out8!("a"));
        // Test non-zero result (A > n)
        m.clear_flags()
            .set(A(0b1000_0000))
            .set_data(&[0b0001_0000])
            .check_result(0b0111_0000, f!(0, 1, 0, 0), out8!("a"));
        // Test half carry (A & 0x0F < n & 0x0F)
        m.clear_flags()
            .set(A(0b0001_0000))
            .set_data(&[0b0000_0001])
            .check_result(0b0000_1111, f!(0, 1, 1, 0), out8!("a"));
        // Test carry flag (A < n) = 0b1111_1111
        m.clear_flags()
            .set(A(0b0000_0001))
            .set_data(&[0b0000_0010])
            .check_result(0b1111_1111, f!(0, 1, 1, 1), out8!("a"));
        // Test carry flag (A < n) = 0b1111_0000
        m.clear_flags()
            .set(A(0b0000_0000))
            .set_data(&[0b0001_0000])
            .check_result(0b1111_0000, f!(0, 1, 0, 1), out8!("a"));
    }

    #[test]
    fn test_add() {
        let mut m = TestMachine::with_operation(ADD(z!("A"), z!("n")));

        // Test zero flag (A == n)
        m.clear_flags()
            .set(A(0x00))
            .set_data(&[0x00])
            .check_result(0x00, f!(1, 0, 0, 0), out8!("a"));
        // Test non-zero result (A + n < 256)
        m.clear_flags()
            .set(A(0b1000_0000))
            .set_data(&[0b0001_0000])
            .check_result(0b1001_0000, f!(0, 0, 0, 0), out8!("a"));
        // Test half carry
        m.clear_flags()
            .set(A(0b0000_1111))
            .set_data(&[0b0000_0001])
            .check_result(0b0001_0000, f!(0, 0, 1, 0), out8!("a"));
        // Test carry flag
        m.clear_flags()
            .set(A(0b1111_0000))
            .set_data(&[0b0001_0010])
            .check_result(0b0000_0010, f!(0, 0, 0, 1), out8!("a"));
        // Test carry and half flags
        m.clear_flags()
            .set(A(0b1100_1100))
            .set_data(&[0b0111_0111])
            .check_result(0b0100_0011, f!(0, 0, 1, 1), out8!("a"));
    }

    #[test]
    fn test_scf() {
        let mut m = TestMachine::with_operation(SCF);

        // Case 1: All flags are erased.
        m.clear_flags().check_flags(f!(0, 0, 0, 1));

        // Case 2: The Z flag is active
        m.clear_flags().set_flags(Flags::Z).check_flags(f!(1, 0, 0, 1));
    }

    #[test]
    fn test_ccf() {
        let mut m = TestMachine::with_operation(CCF);

        // Case 1: Carry flag is unset, should be set
        m.clear_flags().check_flags(f!(0, 0, 0, 1));

        // Case 2: Carry flag is set, should be unset
        m.clear_flags().set_flags(Flags::C).check_flags(f!(0, 0, 0, 0));

        // Case 3: Z flag should remain unchanged when set
        m.clear_flags().set_flags(Flags::Z).check_flags(f!(1, 0, 0, 1));

        // Case 4: Z flag should remain unchanged when set with C
        m.clear_flags()
            .set_flags(Flags::Z | Flags::C)
            .check_flags(f!(1, 0, 0, 0));
    }

    #[test]
    fn test_daa() {
        let mut m = TestMachine::with_operation(DAA);

        // --- Tests after addition (N=0) ---

        // Normal case (without adjustment)
        m.clear_flags()
            .set(A(0x45))
            .set_data(&[0x00])
            .check_result(0x45, f!(0, 0, 0, 0), out8!("a"));

        // Lower nibble value > 9
        m.clear_flags()
            .set(A(0x4A))
            .set_data(&[0x00])
            .check_result(0x50, f!(0, 0, 0, 0), out8!("a"));

        // Case with H flag defined
        m.clear_flags()
            .set_flags(Flags::H)
            .set(A(0x45))
            .set_data(&[0x00])
            .check_result(0x4b, f!(0, 0, 0, 0), out8!("a"));

        // Case with value > 0x99
        m.clear_flags()
            .set(A(0xA5))
            .set_data(&[0x00])
            .check_result(0x05, f!(0, 0, 0, 1), out8!("a"));

        // Case with C flag set
        m.clear_flags()
            .set_flags(Flags::C)
            .set(A(0x45))
            .set_data(&[0x00])
            .check_result(0xA5, f!(0, 0, 0, 1), out8!("a"));

        // Special case - zero result
        m.clear_flags()
            .set(A(0x00))
            .set_data(&[0x00])
            .check_result(0x00, f!(1, 0, 0, 0), out8!("a"));

        // Special case - result 0x9A becoming 0x00 with carry
        m.clear_flags()
            .set(A(0x9A))
            .set_data(&[0x00])
            .check_result(0x00, f!(1, 0, 0, 1), out8!("a"));

        // --- Tests after subtraction (N=1) ---

        // Normal case (without adjustment)
        m.clear_flags()
            .set_flags(Flags::N)
            .set(A(0x45))
            .set_data(&[0x00])
            .check_result(0x45, f!(0, 1, 0, 0), out8!("a"));

        // Case with H flag defined
        m.clear_flags()
            .set_flags(Flags::N | Flags::H)
            .set(A(0x45))
            .set_data(&[0x00])
            .check_result(0x3F, f!(0, 1, 0, 0), out8!("a"));

        // Case with flag C defined
        m.clear_flags()
            .set_flags(Flags::N | Flags::C)
            .set(A(0x45))
            .set_data(&[0x00])
            .check_result(0xE5, f!(0, 1, 0, 1), out8!("a"));

        // Case with H and C flags set
        m.clear_flags()
            .set_flags(Flags::N | Flags::H | Flags::C)
            .set(A(0x45))
            .set_data(&[0x00])
            .check_result(0xDF, f!(0, 1, 0, 1), out8!("a"));
    }

    #[test]
    fn test_add_sp_plus_e() {
        let mut m = TestMachine::with_operation(ADD(z!("SP"), z!("e")));

        // Test normal addition (no flags)
        m.clear_flags()
            .set(SP(0x1000))
            .set_data(&[0x01])
            .check_result(0x1001, f!(0, 0, 0, 0), out16!("sp"));

        // Test half carry flag
        m.clear_flags()
            .set(SP(0x000F))
            .set_data(&[0x01])
            .check_result(0x0010, f!(0, 0, 1, 0), out16!("sp"));

        // Test carry flag
        m.clear_flags()
            .set(SP(0x00F0))
            .set_data(&[0x10])
            .check_result(0x0100, f!(0, 0, 0, 1), out16!("sp"));

        // Test negative offset
        m.clear_flags()
            .set(SP(0x1000))
            .set_data(&[0xFF]) // -1 in two's complement
            .check_result(0x0FFF, f!(0, 0, 0, 0), out16!("sp"));
        m.clear_flags()
            .set(SP(0x0000))
            .set_data(&[0x80])
            .check_result(0xff80, f!(0, 0, 0, 0), out16!("sp"));
    }

    #[test]
    fn test_ld_hl_sp_plus_e() {
        let mut m = TestMachine::with_operation(ADD(z!("HL"), z!("SP+e")));

        // Test zero offset
        m.clear_flags()
            .set(HL(0x0000))
            .set(SP(0x1000))
            .set_data(&[0x00])
            .check_flags(f!(0, 0, 0, 0));
        assert_eq!(0x1000, m.cpu.hl());
        assert_eq!(0x1000, m.cpu.sp());

        // Test positive offset with half carry
        m.clear_flags()
            .set(HL(0x0FF0))
            .set(SP(0xA000))
            .set_data(&[0x10])
            .check_flags(f!(0, 0, 1, 0));
        assert_eq!(0xB000, m.cpu.hl());
        assert_eq!(0xA000, m.cpu.sp());

        // Test positive offset with carry
        m.clear_flags()
            .set(HL(0xF000))
            .set(SP(0x0FF0))
            .set_data(&[0x10])
            .check_flags(f!(0, 0, 0, 1));
        assert_eq!(0x0000, m.cpu.hl());
        assert_eq!(0x0FF0, m.cpu.sp());

        // Test positive offset with both carries
        m.clear_flags()
            .set(HL(0xFF00))
            .set(SP(0x00FF))
            .set_data(&[0x01])
            .check_flags(f!(0, 0, 1, 1));
        assert_eq!(0x0000, m.cpu.hl());
        assert_eq!(0x00FF, m.cpu.sp());
    }

    #[test]
    fn test_cb_rlc() {
        let mut m = TestMachine::with_operation_cb(RLC(z!("B")));

        // Test basic rotation without carry
        m.clear_flags()
            .set(B(0b0100_0000))
            .check_result(0b1000_0000, f!(0, 0, 0, 0), out8!("b"));

        // Test rotation with carry set
        m.clear_flags()
            .set(B(0b1000_0000))
            .check_result(0b0000_0001, f!(0, 0, 0, 1), out8!("b"));

        // Test zero result
        m.clear_flags()
            .set(B(0b0000_0000))
            .check_result(0b0000_0000, f!(1, 0, 0, 0), out8!("b"));

        // Test rotation preserving bits
        m.clear_flags()
            .set(B(0b1010_1010))
            .check_result(0b0101_0101, f!(0, 0, 0, 1), out8!("b"));
    }

    #[test]
    fn test_cb_rrc() {
        let mut m = TestMachine::with_operation_cb(RRC(z!("B")));

        // Test basic rotation without carry
        m.clear_flags()
            .set(B(0b0100_0000))
            .check_result(0b0010_0000, f!(0, 0, 0, 0), out8!("b"));

        // Test rotation with carry set
        m.clear_flags()
            .set(B(0b0000_0001))
            .check_result(0b1000_0000, f!(0, 0, 0, 1), out8!("b"));

        // Test zero result
        m.clear_flags()
            .set(B(0b0000_0000))
            .check_result(0b0000_0000, f!(1, 0, 0, 0), out8!("b"));

        // Test rotation preserving bits
        m.clear_flags()
            .set(B(0b0101_0101))
            .check_result(0b101_01010, f!(0, 0, 0, 1), out8!("b"));
    }

    #[test]
    fn test_cb_rl() {
        let mut m = TestMachine::with_operation_cb(RL(z!("B")));

        // Test basic rotation without carry
        m.clear_flags()
            .set(B(0b0100_0000))
            .check_result(0b1000_0000, f!(0, 0, 0, 0), out8!("b"));

        // Test rotation with previous carry
        m.clear_flags()
            .set_flags(Flags::C)
            .set(B(0b0100_0000))
            .check_result(0b1000_0001, f!(0, 0, 0, 0), out8!("b"));

        // Test rotation setting carry
        m.clear_flags()
            .set(B(0b1000_0000))
            .check_result(0b0000_0000, f!(1, 0, 0, 1), out8!("b"));

        // Test zero result without carry
        m.clear_flags()
            .set(B(0b0000_0000))
            .check_result(0b0000_0000, f!(1, 0, 0, 0), out8!("b"));
    }

    #[test]
    fn test_cb_rr() {
        let mut m = TestMachine::with_operation_cb(RR(z!("B")));

        // Test basic rotation without carry
        m.clear_flags()
            .set(B(0b0000_0010))
            .check_result(0b0000_0001, f!(0, 0, 0, 0), out8!("b"));

        // Test rotation with previous carry
        m.clear_flags()
            .set_flags(Flags::C)
            .set(B(0b0000_0010))
            .check_result(0b1000_0001, f!(0, 0, 0, 0), out8!("b"));

        // Test rotation setting carry
        m.clear_flags()
            .set(B(0b0000_0001))
            .check_result(0b0000_0000, f!(1, 0, 0, 1), out8!("b"));

        // Test zero result without carry
        m.clear_flags()
            .set(B(0b0000_0000))
            .check_result(0b0000_0000, f!(1, 0, 0, 0), out8!("b"));
    }

    #[test]
    fn test_cb_sla() {
        let mut m = TestMachine::with_operation_cb(SLA(z!("B")));

        // Test basic shift left
        m.clear_flags()
            .set(B(0b0100_0000))
            .check_result(0b1000_0000, f!(0, 0, 0, 0), out8!("b"));

        // Test shift left with carry
        m.clear_flags()
            .set(B(0b1000_0000))
            .check_result(0b0000_0000, f!(1, 0, 0, 1), out8!("b"));

        // Test zero result
        m.clear_flags()
            .set(B(0b0000_0000))
            .check_result(0b0000_0000, f!(1, 0, 0, 0), out8!("b"));

        // Test shift preserving bits
        m.clear_flags()
            .set(B(0b0101_0101))
            .check_result(0b1010_1010, f!(0, 0, 0, 0), out8!("b"));
    }

    #[test]
    fn test_cb_sra() {
        let mut m = TestMachine::with_operation_cb(SRA(z!("B")));

        // Test basic shift right preserving sign bit
        m.clear_flags()
            .set(B(0b1100_0000))
            .check_result(0b1110_0000, f!(0, 0, 0, 0), out8!("b"));

        // Test shift right with carry
        m.clear_flags()
            .set_flags(Flags::C)
            .set(B(0b0000_0001))
            .check_result(0b0000_0000, f!(1, 0, 0, 1), out8!("b"));

        // Test shift right preserving sign bit (negative)
        m.clear_flags()
            .set(B(0b1000_0001))
            .check_result(0b1100_0000, f!(0, 0, 0, 1), out8!("b"));

        // Test shift right preserving sign bit (positive)
        m.clear_flags()
            .set(B(0b0100_0001))
            .check_result(0b0010_0000, f!(0, 0, 0, 1), out8!("b"));
    }

    #[test]
    fn test_cb_srl() {
        let mut m = TestMachine::with_operation_cb(SRL(z!("B")));

        // Test basic shift right
        m.clear_flags()
            .set(B(0b1100_0000))
            .check_result(0b0110_0000, f!(0, 0, 0, 0), out8!("b"));

        // Test shift right with carry
        m.clear_flags()
            .set(B(0b0000_0001))
            .check_result(0b0000_0000, f!(1, 0, 0, 1), out8!("b"));

        // Test zero result
        m.clear_flags()
            .set(B(0b0000_0000))
            .check_result(0b0000_0000, f!(1, 0, 0, 0), out8!("b"));

        // Test shift preserving bits
        m.clear_flags()
            .set(B(0b1010_1010))
            .check_result(0b0101_0101, f!(0, 0, 0, 0), out8!("b"));
    }
    #[test]
    fn test_cb_swap() {
        let mut m = TestMachine::with_operation_cb(SWAP(z!("B")));

        // Test basic swap
        m.clear_flags()
            .set(B(0x12))
            .check_result(0x21, f!(0, 0, 0, 0), out8!("b"));

        // Test zero result
        m.clear_flags()
            .set(B(0x00))
            .check_result(0x00, f!(1, 0, 0, 0), out8!("b"));

        // Test swap preserving all bits
        m.clear_flags()
            .set(B(0xF0))
            .check_result(0x0F, f!(0, 0, 0, 0), out8!("b"));

        // Test full byte swap
        m.clear_flags()
            .set(B(0xAB))
            .check_result(0xBA, f!(0, 0, 0, 0), out8!("b"));
    }

    #[test]
    fn test_cb_bit() {
        let mut m = TestMachine::with_operation_cb(BIT(7, z!("B")));

        // Test when bit is set
        m.clear_flags()
            .set(B(0b1100_0001))
            .check_result(0b1100_0001, f!(0, 0, 1, 0), out8!("b"));

        // Test when bit is not set
        m.clear_flags()
            .set(B(0b0100_0000))
            .check_result(0b0100_0000, f!(1, 0, 1, 0), out8!("b"));
    }

    #[test]
    fn test_cb_res() {
        let mut m = TestMachine::with_operation_cb(RES(0, z!("B")));

        // Test resetting bit that is set
        m.clear_flags()
            .set(B(0b0000_0001))
            .check_result(0b0000_0000, f!(0, 0, 0, 0), out8!("b"));

        // Test resetting bit that is already reset
        m.clear_flags()
            .set(B(0b1111_1110))
            .check_result(0b1111_1110, f!(0, 0, 0, 0), out8!("b"));

        // Test resetting bit preserves other bits
        m.clear_flags()
            .set(B(0b1111_1111))
            .check_result(0b1111_1110, f!(0, 0, 0, 0), out8!("b"));
    }

    #[test]
    fn test_cb_set() {
        let mut m = TestMachine::with_operation_cb(SET(0, z!("B")));

        // Test setting bit that is reset
        m.clear_flags()
            .set(B(0b0000_0000))
            .check_result(0b0000_0001, f!(0, 0, 0, 0), out8!("b"));

        // Test setting bit that is already set
        m.clear_flags()
            .set(B(0b0000_0001))
            .check_result(0b0000_0001, f!(0, 0, 0, 0), out8!("b"));

        // Test setting bit preserves other bits
        m.clear_flags()
            .set(B(0b1111_1110))
            .check_result(0b1111_1111, f!(0, 0, 0, 0), out8!("b"));
    }
}
