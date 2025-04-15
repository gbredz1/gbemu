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
    NC, C
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
    Relative,                           // d -> PC+d
    Indirect,                           // (n) -> (0xFF00+n)
    Extended,                           // (nn)
    Register(Reg),                      // X
    RegisterPostIncrement(Reg),          // C+
    RegisterIndirect(Reg),              // (C)
    RegisterIndirectPostIncrement(Reg), // (C)+
    RegisterIndirectPostDecrement(Reg), // (C)-
    RegisterIndirectDec(Reg),           // (C)
}
pub use AddressingMode as AddMod;

#[macro_export]
macro_rules! z {
    ($y:expr) => {
        match ($y) {
            "A" => AddMod::Register(Reg::A),
            "F" => AddMod::Register(Reg::F),
            "B" => AddMod::Register(Reg::B),
            "C" => AddMod::Register(Reg::C),
            "D" => AddMod::Register(Reg::D),
            "E" => AddMod::Register(Reg::E),
            "H" => AddMod::Register(Reg::H),
            "L" => AddMod::Register(Reg::L),
            "AF" => AddMod::Register(Reg::AF),
            "BC" => AddMod::Register(Reg::BC),
            "DE" => AddMod::Register(Reg::DE),
            "HL" => AddMod::Register(Reg::HL),
            "SP" => AddMod::Register(Reg::SP),
            "SP+d" => AddMod::RegisterPostIncrement(Reg::SP),

            "d" => AddMod::Relative,
            "n" => AddMod::Immediate,
            "nn" => AddMod::ImmediateExtended,

            "(AF)" => AddMod::RegisterIndirect(Reg::AF),
            "(BC)" => AddMod::RegisterIndirect(Reg::BC),
            "(DE)" => AddMod::RegisterIndirect(Reg::DE),
            "(HL)" => AddMod::RegisterIndirect(Reg::HL),
            "(HL+)" => AddMod::RegisterIndirectPostIncrement(Reg::HL),
            "(HL-)" => AddMod::RegisterIndirectPostDecrement(Reg::HL),
            "(SP)" => AddMod::RegisterIndirect(Reg::SP),
            "(nn)" => AddMod::Extended,
            "(n)" => AddMod::Indirect,
            "(C)" => AddMod::RegisterIndirect(Reg::C),

            a => panic!("`{a:}` invalid"),
        }
    };
}
#[macro_export]
macro_rules! z_r {
    ($y:expr) => {
        match $y {
            0 => AddMod::Register(Reg::B),
            1 => AddMod::Register(Reg::C),
            2 => AddMod::Register(Reg::D),
            3 => AddMod::Register(Reg::E),
            4 => AddMod::Register(Reg::H),
            5 => AddMod::Register(Reg::L),
            6 => AddMod::RegisterIndirect(Reg::HL),
            7 => AddMod::Register(Reg::A),
            a => panic!("r: `{a:}` invalid must be in [0..7]"),
        }
    };
}
#[macro_export]
macro_rules! z_rp {
    ($y:expr) => {
        match $y {
            0 => AddMod::Register(Reg::BC),
            1 => AddMod::Register(Reg::DE),
            2 => AddMod::Register(Reg::HL),
            3 => AddMod::Register(Reg::SP),
            a => panic!("rp: `{a:}` invalid must be in [0..3]"),
        }
    };
}
#[macro_export]
macro_rules! z_rp2 {
    ($y:expr) => {
        match $y {
            0 => AddMod::Register(Reg::BC),
            1 => AddMod::Register(Reg::DE),
            2 => AddMod::Register(Reg::HL),
            3 => AddMod::Register(Reg::AF),
            a => panic!("rp2: `{a:}` invalid must be in [0..3]"),
        }
    };
}
