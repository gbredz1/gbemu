#[rustfmt::skip]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Register {
    A, F, AF,
    B, C, BC,
    D, E, DE,
    H, L, HL,
    SP,
}
pub use Register as Reg;

#[rustfmt::skip]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Condition {
    NZ, Z,
    NC, C,
}
pub use Condition as CC;

#[macro_export]
macro_rules! z_cc {
    ($y:expr) => {
        match ($y) {
            0 => CC::NZ,
            1 => CC::Z,
            2 => CC::NC,
            3 => CC::C,
            a => panic!("CC: {a:} invalid must be in [0..3]"),
        }
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AddressingMode {
    Immediate,                          // n
    ImmediateExtended,                  // nn
    Relative,                           // d -> PC+e
    Indirect,                           // (n) -> (0xFF00+n)
    Extended,                           // (nn)
    Register(Reg),                      // X
    RegisterIndirect(Reg),              // (C)
    RegisterIndirectPostIncrement(Reg), // (C)+
    RegisterIndirectPostDecrement(Reg), // (C)-
    AdjustedStackPointer,               // SP+e
}
pub use AddressingMode as Op; // Operand

#[macro_export]
macro_rules! z {
    ($y:expr) => {
        match ($y) {
            "A" => Op::Register(Reg::A),
            "F" => Op::Register(Reg::F),
            "B" => Op::Register(Reg::B),
            "C" => Op::Register(Reg::C),
            "D" => Op::Register(Reg::D),
            "E" => Op::Register(Reg::E),
            "H" => Op::Register(Reg::H),
            "L" => Op::Register(Reg::L),
            "AF" => Op::Register(Reg::AF),
            "BC" => Op::Register(Reg::BC),
            "DE" => Op::Register(Reg::DE),
            "HL" => Op::Register(Reg::HL),
            "SP" => Op::Register(Reg::SP),

            "e" => Op::Relative,
            "n" => Op::Immediate,
            "nn" => Op::ImmediateExtended,

            "(AF)" => Op::RegisterIndirect(Reg::AF),
            "(BC)" => Op::RegisterIndirect(Reg::BC),
            "(DE)" => Op::RegisterIndirect(Reg::DE),
            "(HL)" => Op::RegisterIndirect(Reg::HL),
            "(HL+)" => Op::RegisterIndirectPostIncrement(Reg::HL),
            "(HL-)" => Op::RegisterIndirectPostDecrement(Reg::HL),
            "(SP)" => Op::RegisterIndirect(Reg::SP),
            "(nn)" => Op::Extended,
            "(n)" => Op::Indirect,
            "(C)" => Op::RegisterIndirect(Reg::C),

            "SP+e" => Op::AdjustedStackPointer,

            a => panic!("`{a:}` invalid"),
        }
    };
}
#[macro_export]
macro_rules! z_r {
    ($y:expr) => {
        match $y {
            0 => Op::Register(Reg::B),
            1 => Op::Register(Reg::C),
            2 => Op::Register(Reg::D),
            3 => Op::Register(Reg::E),
            4 => Op::Register(Reg::H),
            5 => Op::Register(Reg::L),
            7 => Op::Register(Reg::A),
            a => panic!("r: `{a:}` invalid must be in [0..7] and not equal to 6"),
        }
    };
}
#[macro_export]
macro_rules! z_rp {
    ($y:expr) => {
        match $y {
            0 => Op::Register(Reg::BC),
            1 => Op::Register(Reg::DE),
            2 => Op::Register(Reg::HL),
            3 => Op::Register(Reg::SP),
            a => panic!("rp: `{a:}` invalid must be in [0..3]"),
        }
    };
}
#[macro_export]
macro_rules! z_rp2 {
    ($y:expr) => {
        match $y {
            0 => Op::Register(Reg::BC),
            1 => Op::Register(Reg::DE),
            2 => Op::Register(Reg::HL),
            3 => Op::Register(Reg::AF),
            a => panic!("rp2: `{a:}` invalid must be in [0..3]"),
        }
    };
}
