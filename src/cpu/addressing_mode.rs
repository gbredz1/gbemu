#[rustfmt::skip]
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
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
pub use AddressingMode as Op;

// Operands
#[macro_export]
macro_rules! z {
    ("A") => {
        Op::Register(Reg::A)
    };
    ("B") => {
        Op::Register(Reg::B)
    };
    ("C") => {
        Op::Register(Reg::C)
    };
    ("D") => {
        Op::Register(Reg::D)
    };
    ("E") => {
        Op::Register(Reg::E)
    };
    ("F") => {
        Op::Register(Reg::F)
    };
    ("H") => {
        Op::Register(Reg::H)
    };
    ("L" ) => {
        Op::Register(Reg::L)
    };
    ("AF") => {
        Op::Register(Reg::AF)
    };
    ("BC") => {
        Op::Register(Reg::BC)
    };
    ("DE") => {
        Op::Register(Reg::DE)
    };
    ("HL") => {
        Op::Register(Reg::HL)
    };
    ("SP") => {
        Op::Register(Reg::SP)
    };
    ("e") => {
        Op::Relative
    };
    ("n") => {
        Op::Immediate
    };
    ("nn") => {
        Op::ImmediateExtended
    };
    ("(AF)") => {
        Op::RegisterIndirect(Reg::AF)
    };
    ("(BC)") => {
        Op::RegisterIndirect(Reg::BC)
    };
    ("(DE)") => {
        Op::RegisterIndirect(Reg::DE)
    };
    ("(HL)") => {
        Op::RegisterIndirect(Reg::HL)
    };
    ("(HL+)") => {
        Op::RegisterIndirectPostIncrement(Reg::HL)
    };
    ("(HL-)") => {
        Op::RegisterIndirectPostDecrement(Reg::HL)
    };
    ("(SP)") => {
        Op::RegisterIndirect(Reg::SP)
    };
    ("(nn)") => {
        Op::Extended
    };
    ("(n)") => {
        Op::Indirect
    };
    ("(C)") => {
        Op::RegisterIndirect(Reg::C)
    };
    ("SP+e") => {
        Op::AdjustedStackPointer
    };
}
