use crate::cpu::addressing_mode::{CC, Op};

#[derive(Debug, PartialEq)]
pub enum Operation {
    ADD(Op, Op),
    AND(Op),
    CALL(Op),
    CALLcc(CC, Op),
    BIT(usize, Op),
    CBPrefix,
    CCF,
    CP(Op),
    CPL,
    DAA,
    DEC(Op),
    DI,
    EI,
    HALT,
    INC(Op),
    JP(Op),
    JPcc(CC, Op),
    JR(Op),
    JRcc(CC, Op),
    LD(Op, Op),
    LDH(Op, Op),
    NOP,
    OR(Op),
    POP(Op),
    PUSH(Op),
    RES(usize, Op),
    RET,
    RETcc(CC),
    RETI,
    RL(Op),
    RLA,
    RLC(Op),
    RLCA,
    RR(Op),
    RRA,
    RRC(Op),
    RRCA,
    RST(usize),
    SBC(Op, Op),
    SCF,
    SET(usize, Op),
    SLA(Op),
    SRA(Op),
    SRL(Op),
    STOP,
    SUB(Op),
    SWAP(Op),
    XOR(Op),
    ADC(Op, Op),
}

#[derive(Debug, PartialEq)]
pub struct Instruction {
    pub(crate) operation: Operation,
    pub(crate) size: usize,
    pub(crate) cycles: usize,
    pub(crate) cycles_not_taken: usize,
}

impl Instruction {
    pub(crate) fn from(operation: Operation, size: usize, cycles: usize, cycles_not_taken: usize) -> Self {
        Self {
            operation,
            size,
            cycles,
            cycles_not_taken,
        }
    }
}
