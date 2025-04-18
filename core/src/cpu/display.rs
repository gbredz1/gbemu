use crate::cpu::addressing_mode::{AddressingMode, Register, CC};
use crate::cpu::instruction::Operation;
use std::fmt;
use std::fmt::{Display, Formatter};
use AddressingMode::*;
use Operation::*;

impl Display for Operation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ADC(o1, o2) => write!(f, "ADC {},{}", o1, o2),
            ADD(o1, o2) => write!(f, "ADD {},{}", o1, o2),
            AND(o) => write!(f, "AND {}", o),
            BIT(bit, o) => write!(f, "BIT {},{}", bit, o),
            CALL(o) => write!(f, "CALL {}", o),
            CALLcc(cc, o) => write!(f, "CALL {},{}", cc, o),
            CBPrefix => write!(f, "CB prefix"),
            CCF => write!(f, "CCF"),
            CP(o) => write!(f, "CP {}", o),
            CPL => write!(f, "CPL"),
            DAA => write!(f, "DAA"),
            DEC(o) => write!(f, "DEC {}", o),
            DI => write!(f, "DI"),
            EI => write!(f, "EI"),
            HALT => write!(f, "HALT"),
            INC(o) => write!(f, "INC {}", o),
            JP(o) => write!(f, "JP {}", o),
            JPcc(cc, o) => write!(f, "JP {},{}", cc, o),
            JR(o) => write!(f, "JR {}", o),
            JRcc(cc, o) => write!(f, "JR {},{}", cc, o),
            LD(o1, o2) => write!(f, "LD {},{}", o1, o2),
            LDH(o1, o2) => write!(f, "LDH {},{}", o1, o2),
            NOP => write!(f, "NOP"),
            OR(o) => write!(f, "OR {}", o),
            POP(o) => write!(f, "POP {}", o),
            PUSH(o) => write!(f, "PUSH {}", o),
            RES(bit, o) => write!(f, "RES {},{}", bit, o),
            RET => write!(f, "RET"),
            RETcc(cc) => write!(f, "RET {}", cc),
            RETI => write!(f, "RETI"),
            RL(o) => write!(f, "RL {}", o),
            RLA => write!(f, "RLA"),
            RLC(o) => write!(f, "RLC {}", o),
            RLCA => write!(f, "RLCA"),
            RR(o) => write!(f, "RR {}", o),
            RRA => write!(f, "RRA"),
            RRC(o) => write!(f, "RRC {}", o),
            RRCA => write!(f, "RRCA"),
            RST(addr) => write!(f, "RST {:02X}H", addr),
            SBC(o1, o2) => write!(f, "SBC {},{}", o1, o2),
            SCF => write!(f, "SCF"),
            SET(bit, o) => write!(f, "SET {},{}", bit, o),
            SLA(o) => write!(f, "SLA {}", o),
            SRA(o) => write!(f, "SRA {}", o),
            SRL(o) => write!(f, "SRL {}", o),
            STOP => write!(f, "STOP"),
            SUB(o) => write!(f, "SUB {}", o),
            SWAP(o) => write!(f, "SWAP {}", o),
            XOR(o) => write!(f, "XOR {}", o),
        }
    }
}

impl Display for AddressingMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AdjustedStackPointer => write!(f, "SP+e"),
            Extended => write!(f, "(nn)"),
            Immediate => write!(f, "n"),
            ImmediateExtended => write!(f, "nn"),
            Indirect => write!(f, "(n)"),
            Register(reg) => write!(f, "{}", reg),
            RegisterIndirect(reg) => write!(f, "({})", reg),
            RegisterIndirectPostDecrement(reg) => write!(f, "({}-)", reg),
            RegisterIndirectPostIncrement(reg) => write!(f, "({}+)", reg),
            Relative => write!(f, "e"),
        }
    }
}

impl Display for CC {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CC::C => write!(f, "C"),
            CC::NC => write!(f, "NC"),
            CC::Z => write!(f, "Z"),
            CC::NZ => write!(f, "NZ"),
        }
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Register::A => write!(f, "A"),
            Register::B => write!(f, "B"),
            Register::C => write!(f, "C"),
            Register::D => write!(f, "D"),
            Register::E => write!(f, "E"),
            Register::H => write!(f, "H"),
            Register::L => write!(f, "L"),
            Register::F => write!(f, "F"),
            Register::AF => write!(f, "AF"),
            Register::BC => write!(f, "BC"),
            Register::DE => write!(f, "DE"),
            Register::HL => write!(f, "HL"),
            Register::SP => write!(f, "SP"),
        }
    }
}
