use crate::cpu::addressing_mode::{AddMod, CC};

#[derive(Debug, PartialEq)]
pub enum Instruction {
    // Arithmetic and Logic Instructions
    ADC(AddMod, AddMod), // 8-bit
    ADD(AddMod, AddMod), // 8-bit & 16-bit
    AND(AddMod),         // 8-bit
    CP(AddMod),          // 8-bit
    DEC(AddMod),         // 8-bit & 16-bit
    INC(AddMod),         // 8-bit & 16-bit
    OR(AddMod),          // 8-bit
    SBC(AddMod, AddMod), // 8-bit
    SUB(AddMod),         // 8-bit
    XOR(AddMod),         // 8-bit
    // Bit Operations Instructions
    BIT(usize, AddMod),
    RES(usize, AddMod),
    SET(usize, AddMod),
    SWAP(AddMod),
    // Bit Shift Instructions
    RL(AddMod),
    RLA,
    RLC(AddMod),
    RLCA,
    RR(AddMod),
    RRA,
    RRC(AddMod),
    RRCA,
    SLA(AddMod),
    SRA(AddMod),
    SRL(AddMod),
    // Load Instructions
    LD(AddMod, AddMod),
    LDH(AddMod, AddMod),
    // Jumps and Subroutines
    CALL(AddMod),
    CALLcc(CC, AddMod),
    JP(AddMod),
    JPcc(CC, AddMod),
    JR(AddMod),
    JRcc(CC, AddMod),
    RET,
    RETcc(CC),
    RETI,
    RST(usize),
    // Stack Operations Instructions
    POP(AddMod),
    PUSH(AddMod),
    // Miscellaneous Instructions
    CCF,
    CPL,
    DAA,
    DI,
    EI,
    HALT,
    NOP,
    SCF,
    STOP,
    CBPrefix,
}
