use crate::cpu::addressing_mode::*;
use crate::cpu::instruction::Instruction;
use crate::cpu::instruction::Instruction::*;
use crate::{z, z_cc, z_r, z_rp, z_rp2};

pub struct InstructionInfo {
    instruction: Instruction,
    size: usize,
    cycles: usize,
    cycles_not_taken: usize,
}

impl InstructionInfo {
    fn from(instruction: Instruction,
            size: usize,
            cycles: usize) -> Self {
        Self {
            instruction,
            size,
            cycles,
            cycles_not_taken: 0,
        }
    }
}

pub struct LR35902Decoder {
    main: [Option<InstructionInfo>; 256],
}
impl Default for LR35902Decoder {
    fn default() -> Self {
        let mut decoder = LR35902Decoder {
            main: [const { None }; 256],
        };

        decoder.initialize_main_table();

        decoder
    }
}

impl LR35902Decoder {
    //     Opcode        http://www.z80.info/decoding.htm
    //                   https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
    // 7 6 5 4 3 2 1 0
    // -x- --y-- --z--   x=[0..3], y=[0..7], x=[0..7]
    //     -p- q         p=[0..3], q=[0..1]
    fn initialize_main_table(&mut self) {
        for m in (0..=0xFFu8).map(DecoderMask::from) {
            self.main[m.opcode as usize] = match (m.x, m.y, m.z, m.p, m.q) {
                // For x=0
                (0, 0, 0, _, _) => Some({ NOP, 0, 0, 0 }),                         // NOP
                (0, 1, 0, _, _) => (Some(LD(z!("nn"), z!("SP"))), 3, 20),     // LD (nn), SP
                (0, 2, 0, _, _) => (Some(STOP), 2, 4),                        // STOP
                (0, 3, 0, _, _) => (Some(JR(z!("d"))), 2, 12),                // JR d
                (0, 4..7, 0, _, _) => Some(JRcc(z_cc!(m.y - 4), z!("d"))),    // JR cc[y-4], d
                (0, _, 1, p, 0) => Some(LD(z_rp!(p), z!("nn"))),              // LD rp[p], nn
                (0, _, 1, p, 1) => Some(ADD(z!("HL"), z_rp!(p))),             // ADD HL, rp[p]
                (0, _, 2, 0, 0) => Some(LD(z!("(BC)"), z!("A"))),             // LD (BC), A
                (0, _, 2, 1, 0) => Some(LD(z!("(DE)"), z!("A"))),             // LD (DE), A
                (0, _, 2, 2, 0) => Some(LD(z!("(HL+)"), z!("A"))),            // LD (HL+), A
                (0, _, 2, 3, 0) => Some(LD(z!("(HL-)"), z!("A"))),            // LD (HL-), A
                (0, _, 2, 0, 1) => Some(LD(z!("A"), z!("(BC)"))),             // LD A, (BC)
                (0, _, 2, 1, 1) => Some(LD(z!("A"), z!("(DE)"))),             // LD A, (DE)
                (0, _, 2, 2, 1) => Some(LD(z!("A"), z!("(HL+)"))),            // LD A, (HL+)
                (0, _, 2, 3, 1) => Some(LD(z!("A"), z!("(HL-)"))),            // LD A, (HL-)
                (0, _, 3, p, 0) => Some(INC(z_rp!(p))),                       // INC rp[p]
                (0, _, 3, p, 1) => Some(DEC(z_rp!(p))),                       // DEC rp[p]
                (0, y, 4, _, _) => Some(INC(z_r!(y))),                        // INC r[y]
                (0, y, 5, _, _) => Some(DEC(z_r!(y))),                        // DEC r[y]
                (0, y, 6, _, _) => Some(LD(z_r!(y), z!("n"))),                // LD r[y], n
                (0, 0, 7, _, _) => Some(RLCA),                                // RLCA
                (0, 1, 7, _, _) => Some(RRCA),                                // RRCA
                (0, 2, 7, _, _) => Some(RLA),                                 // RLA
                (0, 3, 7, _, _) => Some(RRA),                                 // RRA
                (0, 4, 7, _, _) => Some(DAA),                                 // DAA
                (0, 5, 7, _, _) => Some(CPL),                                 // CPL
                (0, 6, 7, _, _) => Some(SCF),                                 // SCF
                (0, 7, 7, _, _) => Some(CCF),                                 // CCF
                (1, y, z, _, _) if y < 6 => Some(LD(z_r!(y), z_r!(z))),       // LD r[y], r[z]
                (1, 6, _, _, _) => Some(HALT),                                // HALT
                (2, 0, z, _, _) => Some(ADD(z!("A"), z_r!(z))),               // ADD A, r[z]
                (2, 1, z, _, _) => Some(ADC(z!("A"), z_r!(z))),               // ADC A, r[z]
                (2, 2, _, _, _) => Some(SUB(z!("A"))),                        // SUB A
                (2, 3, z, _, _) => Some(SBC(z!("A"), z_r!(z))),               // SBC A, r[z]
                (2, 4, _, _, _) => Some(AND(z!("A"))),                        // AND A
                (2, 5, _, _, _) => Some(XOR(z!("A"))),                        // XOR A
                (2, 6, _, _, _) => Some(OR(z!("A"))),                         // OR A
                (2, 7, _, _, _) => Some(CP(z!("A"))),                         // CP A
                (3, y, 0, _, _) if y < 4 => Some(RETcc(z_cc!(y))),            // RET cc[y]
                (3, 4, 0, _, _) => Some(LD(z!("(n)"), z!("A"))),              // LD (0xFF00 + n), A
                (3, 5, 0, _, _) => Some(ADD(z!("SP"), z!("d"))),              // ADD SP, d
                (3, 6, 0, _, _) => Some(LD(z!("A"), z!("(n)"))),              // LD A, (0xFF00 + n)
                (3, 7, 0, _, _) => Some(LD(z!("HL"), z!("SP+d"))),            // LD HL, SP+d
                (3, _, 1, 0, p) => Some(POP(z_rp2!(p))),                      // POP rp2[p]
                (3, _, 1, 1, 0) => Some(RET),                                 // RET
                (3, _, 1, 1, 1) => Some(RETI),                                // RETI
                (3, _, 1, 1, 2) => Some(JP(z!("HL"))),                        // JP HL
                (3, _, 1, 1, 3) => Some(LD(z!("SP"), z!("HL"))),              // LD SP,HL
                (3, y, 2, _, _) if y < 4 => Some(JPcc(z_cc!(y), z!("nn"))),   //	JP cc[y], nn
                (3, 4, 2, _, _) => Some(LD(z!("C"), z!("A"))),                // LD (0xFF00+C), A
                (3, 5, 2, _, _) => Some(LD(z!("(nn)"), z!("A"))),             // LD (nn), A
                (3, 6, 2, _, _) => Some(LD(z!("A"), z!("C"))),                // LD A, (0xFF00+C)
                (3, 7, 2, _, _) => Some(LD(z!("A"), z!("(nn)"))),             // LD A, (nn)
                (3, 0, 3, _, _) => Some(JP(z!("nn"))),                        // JP nn
                (3, 1, 3, _, _) => (Some(CBPrefix), 1, 4),                    // (CB prefix)
                (3, 2, 3, _, _) => (None, 0, 0),                              // (removed)
                (3, 3, 3, _, _) => (None, 0, 0),                              // (removed)
                (3, 4, 3, _, _) => (None, 0, 0),                              // (removed)
                (3, 5, 3, _, _) => (None, 0, 0),                              // (removed)
                (3, 6, 3, _, _) => Some(DI),                                  // DI
                (3, 7, 3, _, _) => Some(EI),                                  // EI
                (3, y, 4, _, _) if y < 4 => Some(CALLcc(z_cc![y], z!("nn"))), // CALL cc[y], nn
                (3, _, 4, _, _) => (None, 0, 0),                              // (removed)
                (3, _, 5, 0, p) => Some(PUSH(z_rp2![p])),                     // PUSH rp2[p]
                (3, _, 5, 1, 0) => Some(CALL(z!("nn"))),                      // CALL nn
                (3, _, 5, 1, _) => (None, 0, 0),                              // (removed)
                (3, 0, 6, _, _) => Some(ADD(z!("A"), z!("n"))),               // ADD A, n
                (3, 1, 6, _, _) => Some(ADC(z!("A"), z!("n"))),               // ADC A, n
                (3, 2, 6, _, _) => Some(SUB(z!("n"))),                        // SUB n
                (3, 3, 6, _, _) => Some(SBC(z!("A"), z!("n"))),               // SBC A, n
                (3, 4, 6, _, _) => Some(AND(z!("n"))),                        // AND n
                (3, 5, 6, _, _) => Some(XOR(z!("n"))),                        // XOR n
                (3, 6, 6, _, _) => Some(OR(z!("n"))),                         // OR n
                (3, 7, 6, _, _) => Some(CP(z!("n"))),                         // CP n
                (_, y, 7, _, _) => Some(RST(y * 8)),                          // RST y*8
                // Unknown
                (_, _, _, _, _) => (None, 0, 0),
            }
        }
    }

    fn get_instruction_size(&self, instruction: Instruction) -> usize {
        match instruction {}
    }

    pub(crate) fn decode(&self, opcode: u8) -> &Option<Instruction> {
        &self.main[opcode as usize]
    }
}

struct DecoderMask {
    x: usize,
    y: usize,
    z: usize,
    p: usize,
    q: usize,
    opcode: u8,
}

impl DecoderMask {
    fn from(value: u8) -> Self {
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
