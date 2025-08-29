#[cfg(test)]
pub mod spec_tests {}
#[cfg(test)]
mod tests {
    use crate::cpu::decoder::DecoderMask;
    use crate::cpu_decode;

    #[test]
    fn instruction_decode_test() {
        let mut errors = vec![];
        for &(opcode, _, _, _, desc) in MAIN_INSTR_SPECS.iter() {
            if let Some(instruction) = cpu_decode!(opcode) {
                let str = format!("{}", instruction.operation);
                if str != desc {
                    errors.push(format!(
                        "(0x{:02X}): expected \"{}\" but found \"{}\" -- {}",
                        opcode,
                        desc,
                        str,
                        DecoderMask::from(opcode)
                    ));
                }
            }
        }

        if !errors.is_empty() {
            panic!("Instruction size errors detected:\n{}", errors.join("\n"));
        }
    }

    #[test]
    fn instruction_size_test() {
        let mut errors = vec![];
        for &(opcode, expected_size, _, _, desc) in MAIN_INSTR_SPECS.iter() {
            if let Some(instruction) = cpu_decode!(opcode) {
                if instruction.size != expected_size {
                    errors.push(format!(
                        "!> {} (0x{:02X}): expected size {} but found {} -- {}",
                        desc,
                        opcode,
                        expected_size,
                        instruction.size,
                        DecoderMask::from(opcode)
                    ));
                }
            } else if desc != INVALID_OPCODE_DESC {
                errors.push(format!("@> {} (0x{:02X}): instruction not defined", desc, opcode));
            }
        }

        if !errors.is_empty() {
            panic!("Instruction size errors detected:\n{}", errors.join("\n"));
        }
    }

    #[test]
    fn instruction_timings_test() {
        let mut errors = vec![];
        for &(opcode, _, cycles, cycles_not_taken, desc) in MAIN_INSTR_SPECS.iter() {
            if let Some(instruction) = cpu_decode!(opcode) {
                if instruction.cycles != cycles {
                    errors.push(format!(
                        "!> {} (0x{:02X}): expected cycles {} but found {} -- {}",
                        desc,
                        opcode,
                        cycles,
                        instruction.cycles,
                        DecoderMask::from(opcode)
                    ));
                }
                if instruction.cycles_not_taken != cycles_not_taken {
                    errors.push(format!(
                        "!> {} (0x{:02X}): expected cycles not taken {} but found {}",
                        desc, opcode, cycles_not_taken, instruction.cycles_not_taken
                    ));
                }
            } else if desc != INVALID_OPCODE_DESC {
                errors.push(format!("@> {} (0x{:02X}): instruction not defined", desc, opcode));
            }
        }

        if !errors.is_empty() {
            panic!("Instruction timings errors detected:\n{}", errors.join("\n"));
        }
    }

    const INVALID_OPCODE_DESC: &str = "Invalid opcode";
    // Format: (opcode, size, cycles, cycles_not_taken)
    const MAIN_INSTR_SPECS: [(u8, u8, u8, u8, &str); 256] = [
        // 0x0X
        (0x00, 1, 4, 0, "NOP"),
        (0x01, 3, 12, 0, "LD BC,nn"),
        (0x02, 1, 8, 0, "LD (BC),A"),
        (0x03, 1, 8, 0, "INC BC"),
        (0x04, 1, 4, 0, "INC B"),
        (0x05, 1, 4, 0, "DEC B"),
        (0x06, 2, 8, 0, "LD B,n"),
        (0x07, 1, 4, 0, "RLCA"),
        (0x08, 3, 20, 0, "LD (nn),SP"),
        (0x09, 1, 8, 0, "ADD HL,BC"),
        (0x0A, 1, 8, 0, "LD A,(BC)"),
        (0x0B, 1, 8, 0, "DEC BC"),
        (0x0C, 1, 4, 0, "INC C"),
        (0x0D, 1, 4, 0, "DEC C"),
        (0x0E, 2, 8, 0, "LD C,n"),
        (0x0F, 1, 4, 0, "RRCA"),
        // 0x1X
        (0x10, 1, 4, 0, "STOP"),
        (0x11, 3, 12, 0, "LD DE,nn"),
        (0x12, 1, 8, 0, "LD (DE),A"),
        (0x13, 1, 8, 0, "INC DE"),
        (0x14, 1, 4, 0, "INC D"),
        (0x15, 1, 4, 0, "DEC D"),
        (0x16, 2, 8, 0, "LD D,n"),
        (0x17, 1, 4, 0, "RLA"),
        (0x18, 2, 12, 0, "JR e"),
        (0x19, 1, 8, 0, "ADD HL,DE"),
        (0x1A, 1, 8, 0, "LD A,(DE)"),
        (0x1B, 1, 8, 0, "DEC DE"),
        (0x1C, 1, 4, 0, "INC E"),
        (0x1D, 1, 4, 0, "DEC E"),
        (0x1E, 2, 8, 0, "LD E,n"),
        (0x1F, 1, 4, 0, "RRA"),
        // 0x2X
        (0x20, 2, 12, 8, "JR NZ,e"),
        (0x21, 3, 12, 0, "LD HL,nn"),
        (0x22, 1, 8, 0, "LD (HL+),A"),
        (0x23, 1, 8, 0, "INC HL"),
        (0x24, 1, 4, 0, "INC H"),
        (0x25, 1, 4, 0, "DEC H"),
        (0x26, 2, 8, 0, "LD H,n"),
        (0x27, 1, 4, 0, "DAA"),
        (0x28, 2, 12, 8, "JR Z,e"),
        (0x29, 1, 8, 0, "ADD HL,HL"),
        (0x2A, 1, 8, 0, "LD A,(HL+)"),
        (0x2B, 1, 8, 0, "DEC HL"),
        (0x2C, 1, 4, 0, "INC L"),
        (0x2D, 1, 4, 0, "DEC L"),
        (0x2E, 2, 8, 0, "LD L,n"),
        (0x2F, 1, 4, 0, "CPL"),
        // 0x3X
        (0x30, 2, 12, 8, "JR NC,e"),
        (0x31, 3, 12, 0, "LD SP,nn"),
        (0x32, 1, 8, 0, "LD (HL-),A"),
        (0x33, 1, 8, 0, "INC SP"),
        (0x34, 1, 12, 0, "INC (HL)"),
        (0x35, 1, 12, 0, "DEC (HL)"),
        (0x36, 2, 12, 0, "LD (HL),n"),
        (0x37, 1, 4, 0, "SCF"),
        (0x38, 2, 12, 8, "JR C,e"),
        (0x39, 1, 8, 0, "ADD HL,SP"),
        (0x3A, 1, 8, 0, "LD A,(HL-)"),
        (0x3B, 1, 8, 0, "DEC SP"),
        (0x3C, 1, 4, 0, "INC A"),
        (0x3D, 1, 4, 0, "DEC A"),
        (0x3E, 2, 8, 0, "LD A,n"),
        (0x3F, 1, 4, 0, "CCF"),
        // 0x4X
        (0x40, 1, 4, 0, "LD B,B"),
        (0x41, 1, 4, 0, "LD B,C"),
        (0x42, 1, 4, 0, "LD B,D"),
        (0x43, 1, 4, 0, "LD B,E"),
        (0x44, 1, 4, 0, "LD B,H"),
        (0x45, 1, 4, 0, "LD B,L"),
        (0x46, 1, 8, 0, "LD B,(HL)"),
        (0x47, 1, 4, 0, "LD B,A"),
        (0x48, 1, 4, 0, "LD C,B"),
        (0x49, 1, 4, 0, "LD C,C"),
        (0x4A, 1, 4, 0, "LD C,D"),
        (0x4B, 1, 4, 0, "LD C,E"),
        (0x4C, 1, 4, 0, "LD C,H"),
        (0x4D, 1, 4, 0, "LD C,L"),
        (0x4E, 1, 8, 0, "LD C,(HL)"),
        (0x4F, 1, 4, 0, "LD C,A"),
        // 0x5X
        (0x50, 1, 4, 0, "LD D,B"),
        (0x51, 1, 4, 0, "LD D,C"),
        (0x52, 1, 4, 0, "LD D,D"),
        (0x53, 1, 4, 0, "LD D,E"),
        (0x54, 1, 4, 0, "LD D,H"),
        (0x55, 1, 4, 0, "LD D,L"),
        (0x56, 1, 8, 0, "LD D,(HL)"),
        (0x57, 1, 4, 0, "LD D,A"),
        (0x58, 1, 4, 0, "LD E,B"),
        (0x59, 1, 4, 0, "LD E,C"),
        (0x5A, 1, 4, 0, "LD E,D"),
        (0x5B, 1, 4, 0, "LD E,E"),
        (0x5C, 1, 4, 0, "LD E,H"),
        (0x5D, 1, 4, 0, "LD E,L"),
        (0x5E, 1, 8, 0, "LD E,(HL)"),
        (0x5F, 1, 4, 0, "LD E,A"),
        // 0x6X
        (0x60, 1, 4, 0, "LD H,B"),
        (0x61, 1, 4, 0, "LD H,C"),
        (0x62, 1, 4, 0, "LD H,D"),
        (0x63, 1, 4, 0, "LD H,E"),
        (0x64, 1, 4, 0, "LD H,H"),
        (0x65, 1, 4, 0, "LD H,L"),
        (0x66, 1, 8, 0, "LD H,(HL)"),
        (0x67, 1, 4, 0, "LD H,A"),
        (0x68, 1, 4, 0, "LD L,B"),
        (0x69, 1, 4, 0, "LD L,C"),
        (0x6A, 1, 4, 0, "LD L,D"),
        (0x6B, 1, 4, 0, "LD L,E"),
        (0x6C, 1, 4, 0, "LD L,H"),
        (0x6D, 1, 4, 0, "LD L,L"),
        (0x6E, 1, 8, 0, "LD L,(HL)"),
        (0x6F, 1, 4, 0, "LD L,A"),
        // 0x7X
        (0x70, 1, 8, 0, "LD (HL),B"),
        (0x71, 1, 8, 0, "LD (HL),C"),
        (0x72, 1, 8, 0, "LD (HL),D"),
        (0x73, 1, 8, 0, "LD (HL),E"),
        (0x74, 1, 8, 0, "LD (HL),H"),
        (0x75, 1, 8, 0, "LD (HL),L"),
        (0x76, 1, 4, 0, "HALT"),
        (0x77, 1, 8, 0, "LD (HL),A"),
        (0x78, 1, 4, 0, "LD A,B"),
        (0x79, 1, 4, 0, "LD A,C"),
        (0x7A, 1, 4, 0, "LD A,D"),
        (0x7B, 1, 4, 0, "LD A,E"),
        (0x7C, 1, 4, 0, "LD A,H"),
        (0x7D, 1, 4, 0, "LD A,L"),
        (0x7E, 1, 8, 0, "LD A,(HL)"),
        (0x7F, 1, 4, 0, "LD A,A"),
        // 0x8X
        (0x80, 1, 4, 0, "ADD A,B"),
        (0x81, 1, 4, 0, "ADD A,C"),
        (0x82, 1, 4, 0, "ADD A,D"),
        (0x83, 1, 4, 0, "ADD A,E"),
        (0x84, 1, 4, 0, "ADD A,H"),
        (0x85, 1, 4, 0, "ADD A,L"),
        (0x86, 1, 8, 0, "ADD A,(HL)"),
        (0x87, 1, 4, 0, "ADD A,A"),
        (0x88, 1, 4, 0, "ADC A,B"),
        (0x89, 1, 4, 0, "ADC A,C"),
        (0x8A, 1, 4, 0, "ADC A,D"),
        (0x8B, 1, 4, 0, "ADC A,E"),
        (0x8C, 1, 4, 0, "ADC A,H"),
        (0x8D, 1, 4, 0, "ADC A,L"),
        (0x8E, 1, 8, 0, "ADC A,(HL)"),
        (0x8F, 1, 4, 0, "ADC A,A"),
        // 0x9X
        (0x90, 1, 4, 0, "SUB B"),
        (0x91, 1, 4, 0, "SUB C"),
        (0x92, 1, 4, 0, "SUB D"),
        (0x93, 1, 4, 0, "SUB E"),
        (0x94, 1, 4, 0, "SUB H"),
        (0x95, 1, 4, 0, "SUB L"),
        (0x96, 1, 8, 0, "SUB (HL)"),
        (0x97, 1, 4, 0, "SUB A"),
        (0x98, 1, 4, 0, "SBC A,B"),
        (0x99, 1, 4, 0, "SBC A,C"),
        (0x9A, 1, 4, 0, "SBC A,D"),
        (0x9B, 1, 4, 0, "SBC A,E"),
        (0x9C, 1, 4, 0, "SBC A,H"),
        (0x9D, 1, 4, 0, "SBC A,L"),
        (0x9E, 1, 8, 0, "SBC A,(HL)"),
        (0x9F, 1, 4, 0, "SBC A,A"),
        // 0xAX
        (0xA0, 1, 4, 0, "AND B"),
        (0xA1, 1, 4, 0, "AND C"),
        (0xA2, 1, 4, 0, "AND D"),
        (0xA3, 1, 4, 0, "AND E"),
        (0xA4, 1, 4, 0, "AND H"),
        (0xA5, 1, 4, 0, "AND L"),
        (0xA6, 1, 8, 0, "AND (HL)"),
        (0xA7, 1, 4, 0, "AND A"),
        (0xA8, 1, 4, 0, "XOR B"),
        (0xA9, 1, 4, 0, "XOR C"),
        (0xAA, 1, 4, 0, "XOR D"),
        (0xAB, 1, 4, 0, "XOR E"),
        (0xAC, 1, 4, 0, "XOR H"),
        (0xAD, 1, 4, 0, "XOR L"),
        (0xAE, 1, 8, 0, "XOR (HL)"),
        (0xAF, 1, 4, 0, "XOR A"),
        // 0xBX
        (0xB0, 1, 4, 0, "OR B"),
        (0xB1, 1, 4, 0, "OR C"),
        (0xB2, 1, 4, 0, "OR D"),
        (0xB3, 1, 4, 0, "OR E"),
        (0xB4, 1, 4, 0, "OR H"),
        (0xB5, 1, 4, 0, "OR L"),
        (0xB6, 1, 8, 0, "OR (HL)"),
        (0xB7, 1, 4, 0, "OR A"),
        (0xB8, 1, 4, 0, "CP B"),
        (0xB9, 1, 4, 0, "CP C"),
        (0xBA, 1, 4, 0, "CP D"),
        (0xBB, 1, 4, 0, "CP E"),
        (0xBC, 1, 4, 0, "CP H"),
        (0xBD, 1, 4, 0, "CP L"),
        (0xBE, 1, 8, 0, "CP (HL)"),
        (0xBF, 1, 4, 0, "CP A"),
        // 0xCX
        (0xC0, 1, 20, 8, "RET NZ"),
        (0xC1, 1, 12, 0, "POP BC"),
        (0xC2, 3, 16, 12, "JP NZ,nn"),
        (0xC3, 3, 16, 0, "JP nn"),
        (0xC4, 3, 24, 12, "CALL NZ,nn"),
        (0xC5, 1, 16, 0, "PUSH BC"),
        (0xC6, 2, 8, 0, "ADD A,n"),
        (0xC7, 1, 16, 0, "RST 00H"),
        (0xC8, 1, 20, 8, "RET Z"),
        (0xC9, 1, 16, 0, "RET"),
        (0xCA, 3, 16, 12, "JP Z,nn"),
        (0xCB, 1, 4, 0, "CB prefix"),
        (0xCC, 3, 24, 12, "CALL Z,nn"),
        (0xCD, 3, 24, 0, "CALL nn"),
        (0xCE, 2, 8, 0, "ADC A,n"),
        (0xCF, 1, 16, 0, "RST 08H"),
        // 0xDX
        (0xD0, 1, 20, 8, "RET NC"),
        (0xD1, 1, 12, 0, "POP DE"),
        (0xD2, 3, 16, 12, "JP NC,nn"),
        (0xD3, 1, 0, 0, INVALID_OPCODE_DESC),
        (0xD4, 3, 24, 12, "CALL NC,nn"),
        (0xD5, 1, 16, 0, "PUSH DE"),
        (0xD6, 2, 8, 0, "SUB n"),
        (0xD7, 1, 16, 0, "RST 10H"),
        (0xD8, 1, 20, 8, "RET C"),
        (0xD9, 1, 16, 0, "RETI"),
        (0xDA, 3, 16, 12, "JP C,nn"),
        (0xDB, 1, 0, 0, INVALID_OPCODE_DESC),
        (0xDC, 3, 24, 12, "CALL C,nn"),
        (0xDD, 1, 0, 0, INVALID_OPCODE_DESC),
        (0xDE, 2, 8, 0, "SBC A,n"),
        (0xDF, 1, 16, 0, "RST 18H"),
        // 0xEX
        (0xE0, 2, 12, 0, "LDH (n),A"),
        (0xE1, 1, 12, 0, "POP HL"),
        (0xE2, 1, 8, 0, "LDH (C),A"),
        (0xE3, 1, 0, 0, INVALID_OPCODE_DESC),
        (0xE4, 1, 0, 0, INVALID_OPCODE_DESC),
        (0xE5, 1, 16, 0, "PUSH HL"),
        (0xE6, 2, 8, 0, "AND n"),
        (0xE7, 1, 16, 0, "RST 20H"),
        (0xE8, 2, 16, 0, "ADD SP,e"),
        (0xE9, 1, 4, 0, "JP HL"),
        (0xEA, 3, 16, 0, "LD (nn),A"),
        (0xEB, 1, 0, 0, INVALID_OPCODE_DESC),
        (0xEC, 1, 0, 0, INVALID_OPCODE_DESC),
        (0xED, 1, 0, 0, INVALID_OPCODE_DESC),
        (0xEE, 2, 8, 0, "XOR n"),
        (0xEF, 1, 16, 0, "RST 28H"),
        // 0xFX
        (0xF0, 2, 12, 0, "LDH A,(n)"),
        (0xF1, 1, 12, 0, "POP AF"),
        (0xF2, 1, 8, 0, "LDH A,(C)"),
        (0xF3, 1, 4, 0, "DI"),
        (0xF4, 1, 0, 0, INVALID_OPCODE_DESC),
        (0xF5, 1, 16, 0, "PUSH AF"),
        (0xF6, 2, 8, 0, "OR n"),
        (0xF7, 1, 16, 0, "RST 30H"),
        (0xF8, 2, 12, 0, "LD HL,SP+e"),
        (0xF9, 1, 8, 0, "LD SP,HL"),
        (0xFA, 3, 16, 0, "LD A,(nn)"),
        (0xFB, 1, 4, 0, "EI"),
        (0xFC, 1, 0, 0, INVALID_OPCODE_DESC),
        (0xFD, 1, 0, 0, INVALID_OPCODE_DESC),
        (0xFE, 2, 8, 0, "CP n"),
        (0xFF, 1, 16, 0, "RST 38H"),
    ];
}
