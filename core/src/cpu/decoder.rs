use crate::cpu::addressing_mode::*;
use crate::cpu::instruction::Instruction;
use crate::cpu::instruction::Operation::*;
use crate::z;
use std::fmt::{Display, Formatter};
use std::sync::OnceLock;

#[macro_export]
macro_rules! cpu_decode {
    ($opcode:expr) => {
        $crate::cpu::decoder::LR35902Decoder::decode($opcode)
    };
}

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

macro_rules! instr {
    // Pattern for instructions without cycles_not_taken
    ($inst:expr, $size:expr, $cycles:expr) => {
        Some(Instruction::from($inst, $size, $cycles, 0))
    };

    // Pattern for instructions with all parameters
    ($inst:expr, $size:expr, $cycles:expr, $cycles_not_taken:expr) => {
        Some(Instruction::from($inst, $size, $cycles, $cycles_not_taken))
    };
}

pub(crate) struct LR35902Decoder {}

static MAIN_TABLE: OnceLock<[Option<Instruction>; 256]> = OnceLock::new();
pub fn get_main_table() -> &'static [Option<Instruction>; 256] {
    MAIN_TABLE.get_or_init(LR35902Decoder::build_main_table)
}

impl LR35902Decoder {
    //     Opcode        http://www.z80.info/decoding.htm
    //                   https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
    // 7 6 5 4 3 2 1 0
    // -x- --y-- --z--   x=[0..3], y=[0..7], x=[0..7]
    //     -p- q         p=[0..3], q=[0..1]
    fn build_main_table() -> [Option<Instruction>; 256] {
        let mut table = [const { None }; 256];

        for m in (0..=0xFFu8).map(DecoderMask::from) {
            table[m.opcode as usize] = match (m.x, m.y, m.z, m.p, m.q) {
                (0, 0, 0, _, _) => instr!(NOP, 1, 4),                                      // NOP
                (0, 1, 0, _, _) => instr!(LD(z!("(nn)"), z!("SP")), 3, 20),                // LD (nn),SP
                (0, 2, 0, _, _) => instr!(STOP, 2, 4),                                     // STOP
                (0, 3, 0, _, _) => instr!(JR(z!("e")), 2, 12),                             // JR e
                (0, 4..=7, 0, _, _) => instr!(JRcc(z_cc!(m.y - 4), z!("e")), 2, 12, 8),    // JR cc[y-4],e
                (0, _, 1, p, 0) => instr!(LD(z_rp!(p), z!("nn")), 3, 12),                  // LD rp[p],nn
                (0, _, 1, p, 1) => instr!(ADD(z!("HL"), z_rp!(p)), 1, 8),                  // ADD HL,rp[p]
                (0, _, 2, 0, 0) => instr!(LD(z!("(BC)"), z!("A")), 1, 8),                  // LD (BC),A
                (0, _, 2, 1, 0) => instr!(LD(z!("(DE)"), z!("A")), 1, 8),                  // LD (DE),A
                (0, _, 2, 2, 0) => instr!(LD(z!("(HL+)"), z!("A")), 1, 8),                 // LD (HL+),A
                (0, _, 2, 3, 0) => instr!(LD(z!("(HL-)"), z!("A")), 1, 8),                 // LD (HL-),A
                (0, _, 2, 0, 1) => instr!(LD(z!("A"), z!("(BC)")), 1, 8),                  // LD A,(BC)
                (0, _, 2, 1, 1) => instr!(LD(z!("A"), z!("(DE)")), 1, 8),                  // LD A,(DE)
                (0, _, 2, 2, 1) => instr!(LD(z!("A"), z!("(HL+)")), 1, 8),                 // LD A,(HL+)
                (0, _, 2, 3, 1) => instr!(LD(z!("A"), z!("(HL-)")), 1, 8),                 // LD A,(HL-)
                (0, _, 3, p, 0) => instr!(INC(z_rp!(p)), 1, 8),                            // INC rp[p]
                (0, _, 3, p, 1) => instr!(DEC(z_rp!(p)), 1, 8),                            // DEC rp[p]
                (0, 6, 4, _, _) => instr!(INC(z!("(HL)")), 1, 12),                         // INC (HL)
                (0, y, 4, _, _) => instr!(INC(z_r!(y)), 1, 4),                             // INC r[y]
                (0, 6, 5, _, _) => instr!(DEC(z!("(HL)")), 1, 12),                         // DEC (HL)
                (0, y, 5, _, _) => instr!(DEC(z_r!(y)), 1, 4),                             // DEC r[y]
                (0, 6, 6, _, _) => instr!(LD(z!("(HL)"), z!("n")), 2, 12),                 // LD r[y],n
                (0, y, 6, _, _) => instr!(LD(z_r!(y), z!("n")), 2, 8),                     // LD r[y],n
                (0, 0, 7, _, _) => instr!(RLCA, 1, 4),                                     // RLCA
                (0, 1, 7, _, _) => instr!(RRCA, 1, 4),                                     // RRCA
                (0, 2, 7, _, _) => instr!(RLA, 1, 4),                                      // RLA
                (0, 3, 7, _, _) => instr!(RRA, 1, 4),                                      // RRA
                (0, 4, 7, _, _) => instr!(DAA, 1, 4),                                      // DAA
                (0, 5, 7, _, _) => instr!(CPL, 1, 4),                                      // CPL
                (0, 6, 7, _, _) => instr!(SCF, 1, 4),                                      // SCF
                (0, 7, 7, _, _) => instr!(CCF, 1, 4),                                      // CCF
                (1, y, 6, _, _) if y != 6 => instr!(LD(z_r!(y), z!("(HL)")), 1, 8),        // LD r[y],(HL)
                (1, 6, z, _, _) if z != 6 => instr!(LD(z!("(HL)"), z_r!(z)), 1, 8),        // LD (HL),r[z]
                (1, y, z, _, _) if y != 6 => instr!(LD(z_r!(y), z_r!(z)), 1, 4),           // LD r[y],r[z]
                (1, 6, _, _, _) => instr!(HALT, 1, 4),                                     // HALT
                (2, 0, 6, _, _) => instr!(ADD(z!("A"), z!("(HL)")), 1, 8),                 // ADD A,(HL)
                (2, 0, z, _, _) => instr!(ADD(z!("A"), z_r!(z)), 1, 4),                    // ADD A,r[z]
                (2, 1, 6, _, _) => instr!(ADC(z!("A"), z!("(HL)")), 1, 8),                 // ADC A,r[z]
                (2, 1, z, _, _) => instr!(ADC(z!("A"), z_r!(z)), 1, 4),                    // ADC A,r[z]
                (2, 2, 6, _, _) => instr!(SUB(z!("(HL)")), 1, 8),                          // SUB (HL)
                (2, 2, z, _, _) => instr!(SUB(z_r!(z)), 1, 4),                             // SUB r[z]
                (2, 3, 6, _, _) => instr!(SBC(z!("A"), z!("(HL)")), 1, 8),                 // SBC A,r[z]
                (2, 3, z, _, _) => instr!(SBC(z!("A"), z_r!(z)), 1, 4),                    // SBC A,r[z]
                (2, 4, 6, _, _) => instr!(AND(z!("(HL)")), 1, 8),                          // AND (HL)
                (2, 4, z, _, _) => instr!(AND(z_r!(z)), 1, 4),                             // AND r[z]
                (2, 5, 6, _, _) => instr!(XOR(z!("(HL)")), 1, 8),                          // XOR (HL)
                (2, 5, z, _, _) => instr!(XOR(z_r!(z)), 1, 4),                             // XOR z[z]
                (2, 6, 6, _, _) => instr!(OR(z!("(HL)")), 1, 8),                           // OR (HL)
                (2, 6, z, _, _) => instr!(OR(z_r!(z)), 1, 4),                              // OR z[z]
                (2, 7, 6, _, _) => instr!(CP(z!("(HL)")), 1, 8),                           // CP (HL)
                (2, 7, z, _, _) => instr!(CP(z_r!(z)), 1, 4),                              // CP z[z]
                (3, y, 0, _, _) if y < 4 => instr!(RETcc(z_cc!(y)), 1, 20, 8),             // RET cc[y]
                (3, 4, 0, _, _) => instr!(LDH(z!("(n)"), z!("A")), 2, 12),                 // LDH (n),A
                (3, 5, 0, _, _) => instr!(ADD(z!("SP"), z!("e")), 2, 16),                  // ADD SP, e
                (3, 6, 0, _, _) => instr!(LDH(z!("A"), z!("(n)")), 2, 12),                 // LDH A,(n)
                (3, 7, 0, _, _) => instr!(LD(z!("HL"), z!("SP+e")), 2, 12),                // LD HL,SP+e
                (3, _, 1, p, 0) => instr!(POP(z_rp2!(p)), 1, 12),                          // POP rp2[p]
                (3, _, 1, 0, 1) => instr!(RET, 1, 16),                                     // RET
                (3, _, 1, 1, 1) => instr!(RETI, 1, 16),                                    // RETI
                (3, _, 1, 2, 1) => instr!(JP(z!("(HL)")), 1, 4),                           // JP (HL)
                (3, _, 1, 3, 1) => instr!(LD(z!("SP"), z!("HL")), 1, 8),                   // LD SP,HL
                (3, y, 2, _, _) if y < 4 => instr!(JPcc(z_cc!(y), z!("nn")), 3, 16, 12),   // JP cc[y],nn
                (3, 4, 2, _, _) => instr!(LDH(z!("(C)"), z!("A")), 1, 8),                  // LDH (C),A
                (3, 5, 2, _, _) => instr!(LD(z!("(nn)"), z!("A")), 3, 16),                 // LD (nn),A
                (3, 6, 2, _, _) => instr!(LDH(z!("A"), z!("(C)")), 1, 8),                  // LDH A,(C)
                (3, 7, 2, _, _) => instr!(LD(z!("A"), z!("(nn)")), 3, 16),                 // LD A,(nn)
                (3, 0, 3, _, _) => instr!(JP(z!("nn")), 3, 16),                            // JP nn
                (3, 1, 3, _, _) => instr!(CBPrefix, 1, 4),                                 // (CB prefix)
                (3, 2, 3, _, _) => None,                                                   // (removed)
                (3, 3, 3, _, _) => None,                                                   // (removed)
                (3, 4, 3, _, _) => None,                                                   // (removed)
                (3, 5, 3, _, _) => None,                                                   // (removed)
                (3, 6, 3, _, _) => instr!(DI, 1, 4),                                       // DI
                (3, 7, 3, _, _) => instr!(EI, 1, 4),                                       // EI
                (3, y, 4, _, _) if y < 4 => instr!(CALLcc(z_cc![y], z!("nn")), 3, 24, 12), // CALL cc[y],nn
                (3, _, 4, _, _) => None,                                                   // (removed)
                (3, _, 5, p, 0) => instr!(PUSH(z_rp2![p]), 1, 16),                         // PUSH rp2[p]
                (3, _, 5, 0, 1) => instr!(CALL(z!("nn")), 3, 24),                          // CALL nn
                (3, _, 5, _, 1) => None,                                                   // (removed)
                (3, 0, 6, _, _) => instr!(ADD(z!("A"), z!("n")), 2, 8),                    // ADD A,n
                (3, 1, 6, _, _) => instr!(ADC(z!("A"), z!("n")), 2, 8),                    // ADC A,n
                (3, 2, 6, _, _) => instr!(SUB(z!("n")), 2, 8),                             // SUB n
                (3, 3, 6, _, _) => instr!(SBC(z!("A"), z!("n")), 2, 8),                    // SBC A,n
                (3, 4, 6, _, _) => instr!(AND(z!("n")), 2, 8),                             // AND n
                (3, 5, 6, _, _) => instr!(XOR(z!("n")), 2, 8),                             // XOR n
                (3, 6, 6, _, _) => instr!(OR(z!("n")), 2, 8),                              // OR n
                (3, 7, 6, _, _) => instr!(CP(z!("n")), 2, 8),                              // CP n
                (_, y, 7, _, _) => instr!(RST(y * 8), 1, 16),                              // RST y*8
                // Unknown
                (_, _, _, _, _) => None,
            }
        }

        table
    }

    pub(crate) fn decode(opcode: u8) -> &'static Option<Instruction> {
        let table = get_main_table();

        &table[opcode as usize]
    }
}

#[derive(Debug)]
pub(crate) struct DecoderMask {
    x: usize,
    y: usize,
    z: usize,
    p: usize,
    q: usize,
    opcode: u8,
}

impl Display for DecoderMask {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ {}:{}:{} | {}:{} }}", self.x, self.y, self.z, self.p, self.q)
    }
}

impl DecoderMask {
    pub(crate) fn from(value: u8) -> Self {
        Self {
            x: (value >> 6 & 0x03u8) as usize,
            y: (value >> 3 & 0x07u8) as usize,
            z: (value & 0x07u8) as usize,
            p: (value >> 4 & 0x03u8) as usize,
            q: (value >> 3 & 0x01u8) as usize,
            opcode: value,
        }
    }
}
