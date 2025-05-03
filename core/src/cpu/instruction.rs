use crate::cpu::addressing_mode::Reg;
use crate::cpu::addressing_mode::{CC, Op};
use crate::cpu::instruction::Operation::*;
use crate::cpu::{Cpu, CpuBus, Flags};
use crate::z;
use log::{error, trace};

macro_rules! read_u16_le {
    ($data:expr) => {
        ($data[1] as u16) << 8 | ($data[0] as u16)
    };
    ($data:expr, $index:expr) => {
        ($data[$index + 1] as u16) << 8 | ($data[$index] as u16)
    };
}

macro_rules! read_operand_value_u8 {
    ($cpu:expr, $bus:expr, $data:expr, $op:expr) => {
        match $op {
            z!("A") => $cpu.a(),
            z!("B") => $cpu.b(),
            z!("C") => $cpu.c(),
            z!("D") => $cpu.d(),
            z!("E") => $cpu.e(),
            z!("H") => $cpu.h(),
            z!("L") => $cpu.l(),
            z!("n") => $data[0],
            z!("e") => $data[0],
            z!("(HL)") => $bus.read_byte($cpu.hl()),
            z!("(HL+)") => {
                let value = $bus.read_byte($cpu.hl());
                $cpu.set_hl($cpu.hl().wrapping_add(1));
                value
            }
            z!("(HL-)") => {
                let value = $bus.read_byte($cpu.hl());
                $cpu.set_hl($cpu.hl().wrapping_sub(1));
                value
            }
            z!("(n)") => $bus.read_byte(0xFF00 | $data[0] as u16),
            z!("(C)") => $bus.read_byte(0xFF00 | $cpu.c() as u16),
            z!("(DE)") => $bus.read_byte($cpu.de()),
            z!("(BC)") => $bus.read_byte($cpu.bc()),
            z!("(nn)") => $bus.read_byte(read_u16_le!($data)),
            _ => {
                error!("op_read_u8: Unsupported operand: `{}`", $op);
                unreachable!("Unsupported operand")
            }
        }
    };
}
macro_rules! read_operand_value_u16 {
    ($cpu:expr, $bus:expr, $data:expr, $op:expr) => {
        match $op {
            z!("AF") => $cpu.af(),
            z!("BC") => $cpu.bc(),
            z!("DE") => $cpu.de(),
            z!("nn") => read_u16_le!($data),
            z!("HL") => $cpu.hl(),
            _ => {
                error!("op_read_u16: Unsupported operand: `{}`", $op);
                unreachable!("Unsupported operand")
            }
        }
    };
}
macro_rules! write_to_operand_u8 {
    ($cpu:expr, $bus:expr, $data:expr, $op:expr, $value:expr) => {
        match $op {
            z!("A") => $cpu.set_a($value),
            z!("B") => $cpu.set_b($value),
            z!("C") => $cpu.set_c($value),
            z!("D") => $cpu.set_d($value),
            z!("E") => $cpu.set_e($value),
            z!("H") => $cpu.set_h($value),
            z!("L") => $cpu.set_l($value),
            z!("(n)") => $bus.write_byte(0xFF00 | $data[0] as u16, $value),
            z!("(C)") => $bus.write_byte(0xFF00 | $cpu.c() as u16, $value),
            z!("(nn)") => $bus.write_byte(read_u16_le!($data), $value),
            z!("(DE)") => $bus.write_byte($cpu.de(), $value),
            z!("(BC)") => $bus.write_byte($cpu.bc(), $value),
            z!("(HL)") => $bus.write_byte($cpu.hl(), $value),
            z!("(HL+)") => {
                $bus.write_byte($cpu.hl(), $value);
                $cpu.set_hl($cpu.hl().wrapping_add(1));
            }
            z!("(HL-)") => {
                $bus.write_byte($cpu.hl(), $value);
                $cpu.set_hl($cpu.hl().wrapping_sub(1));
            }
            _ => {
                error!("op_write_u8: Unsupported operand: `{}`", $op);
                unreachable!("Unsupported operand")
            }
        }
    };
}
macro_rules! write_to_operand_u16 {
    ($cpu:expr, $bus:expr, $op:expr, $value:expr) => {
        match $op {
            z!("AF") => $cpu.set_af($value),
            z!("BC") => $cpu.set_bc($value),
            z!("DE") => $cpu.set_de($value),
            z!("HL") => $cpu.set_hl($value),
            z!("SP") => $cpu.set_sp($value),
            _ => {
                error!("op_write_u16: Unsupported operand: `{}`", $op);
                unreachable!("Unsupported operand")
            }
        }
    };
}

macro_rules! handle_cc_not_taken {
    ($self:expr, $cpu:expr, $cc:expr) => {{
        if !$cpu.check_condition($cc) {
            return $self.cycles_not_taken;
        }
    }};
}

macro_rules! match_size {
    ($op1:expr, $block_8bit:block, $block_16bit:block) => {
        match $op1 {
            z!("AF") | z!("BC") | z!("DE") | z!("HL") | z!("SP") => $block_16bit,
            _ => $block_8bit,
        }
    };
    ($cpu:expr, $bus:expr, $data:expr, $op1:expr, $op2:expr, $block_8bit:block) => {
        match $op1 {
            z!("AF") | z!("BC") | z!("DE") | z!("HL") | z!("SP") => {}
            _ => $block_8bit,
        }
    };
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
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

    pub fn execute(&self, cpu: &mut Cpu, bus: &mut impl CpuBus, data: Vec<u8>) -> usize {
        match self.operation {
            NOP => self.cycles,

            JP(op) => {
                let address = read_operand_value_u16!(cpu, bus, data, op);
                trace!("jump to ${:04x}", address);
                cpu.pc = address;

                self.cycles
            }
            JPcc(cc, op) => {
                handle_cc_not_taken!(self, cpu, cc);

                let address = read_operand_value_u16!(cpu, bus, data, op);
                trace!("jump to ${:04x}", address);
                cpu.pc = address;

                self.cycles
            }
            JR(op) => {
                let offset = read_operand_value_u8!(cpu, bus, data, op) as i8; // e
                trace!("jump to ${:04x} {}", cpu.pc(), offset);
                cpu.set_pc(cpu.pc().wrapping_add_signed(offset as i16));

                self.cycles
            }
            JRcc(cc, op) => {
                handle_cc_not_taken!(self, cpu, cc);

                let offset = read_operand_value_u8!(cpu, bus, data, op) as i8; // e
                trace!("jump to ${:04x} {}", cpu.pc(), offset);
                cpu.set_pc(cpu.pc().wrapping_add_signed(offset as i16));

                self.cycles
            }
            CALL(op) => {
                let dest_addr = read_operand_value_u16!(cpu, bus, data, op);
                let return_addr = cpu.pc(); // pc already on next opcode
                cpu.sp_push_word(bus, return_addr);
                cpu.set_pc(dest_addr);

                self.cycles
            }
            CALLcc(cc, op) => {
                handle_cc_not_taken!(self, cpu, cc);

                let dest_addr = read_operand_value_u16!(cpu, bus, data, op);
                let return_addr = cpu.pc(); // pc already on next opcode
                cpu.sp_push_word(bus, return_addr);
                cpu.set_pc(dest_addr);

                self.cycles
            }
            RET => {
                let return_addr = cpu.sp_pop_word(bus);
                cpu.set_pc(return_addr);

                self.cycles
            }
            RETcc(cc) => {
                handle_cc_not_taken!(self, cpu, cc);

                let return_addr = cpu.sp_pop_word(bus);
                cpu.set_pc(return_addr);

                self.cycles
            }
            RETI => {
                let return_addr = cpu.sp_pop_word(bus);
                cpu.set_pc(return_addr);
                cpu.ime = true;

                self.cycles
            }
            PUSH(op) => {
                let value = read_operand_value_u16!(cpu, bus, data, op);
                cpu.sp_push_word(bus, value);

                self.cycles
            }
            POP(op) => {
                let value = cpu.sp_pop_word(bus);
                write_to_operand_u16!(cpu, bus, op, value);

                self.cycles
            }

            AND(op) => {
                let value = read_operand_value_u8!(cpu, bus, data, op);
                cpu.set_a(cpu.a() & value);

                cpu.set_flag_if(Flags::Z, cpu.a() == 0);
                cpu.clear_flag(Flags::N);
                cpu.set_flag(Flags::H);
                cpu.clear_flag(Flags::C);

                self.cycles
            }
            XOR(op) => {
                let value = read_operand_value_u8!(cpu, bus, data, op);
                cpu.set_a(cpu.a() ^ value);

                cpu.set_flag_if(Flags::Z, cpu.a() == 0);
                cpu.clear_flag(Flags::N | Flags::H | Flags::C);

                self.cycles
            }
            ADD(op1, op2) => {
                match_size!(
                    op1,
                    {
                        let val1 = read_operand_value_u8!(cpu, bus, data, op1);
                        let val2 = read_operand_value_u8!(cpu, bus, data, op2);
                        let (result, carry) = val1.overflowing_add(val2);
                        write_to_operand_u8!(cpu, bus, data, op1, result);

                        cpu.set_flag_if(Flags::Z, result == 0);
                        cpu.clear_flag(Flags::N);
                        cpu.set_flag_if(Flags::C, carry);
                        cpu.set_flag_if(Flags::H, (val1 & 0x0F) + (val2 & 0x0F) > 0x0F);
                    },
                    {
                        // 16-bits
                        let val1 = read_operand_value_u16!(cpu, bus, data, op1);

                        if op2 == z!("e") {
                            let val2 = read_operand_value_u8!(cpu, bus, data, op2) as i16;
                            let (result, carry) = val1.overflowing_add_signed(val2);
                            write_to_operand_u16!(cpu, bus, op1, result);

                            cpu.clear_flag(Flags::Z);
                            cpu.clear_flag(Flags::N);
                            cpu.set_flag_if(Flags::H, (val1 & 0x0F) + (val2 as u16 & 0x0F) > 0x0F);
                            cpu.set_flag_if(Flags::C, carry);
                        } else {
                            let val2 = read_operand_value_u16!(cpu, bus, data, op2);
                            let (result, carry) = val1.overflowing_add(val2);
                            write_to_operand_u16!(cpu, bus, op1, result);

                            cpu.clear_flag(Flags::N);
                            cpu.set_flag_if(Flags::H, (val1 & 0x0FFF) + (val2 & 0x0FFF) > 0x0FFF);
                            cpu.set_flag_if(Flags::C, carry);
                        }
                    }
                );

                self.cycles
            }

            ADC(z!("A"), op2) => {
                let carry = if cpu.flag(Flags::C) { 1 } else { 0 };
                let val2 = cpu.a();
                let val1 = read_operand_value_u8!(cpu, bus, data, op2);

                let result = val1.wrapping_add(val2).wrapping_add(carry);
                cpu.set_a(result);

                cpu.set_flag_if(Flags::Z, result == 0);
                cpu.clear_flag(Flags::N);
                cpu.set_flag_if(Flags::H, (val1 & 0x0F) + (val2 & 0x0F) + carry > 0x0F);
                cpu.set_flag_if(Flags::C, val1 as u16 + val2 as u16 + carry as u16 > 0xFF);

                self.cycles
            }
            SUB(op) => {
                let val = read_operand_value_u8!(cpu, bus, data, op);

                let result = cpu.a().wrapping_sub(val);
                cpu.set_a(result);

                cpu.set_flag_if(Flags::Z, result == 0);
                cpu.set_flag(Flags::N);
                cpu.set_flag_if(Flags::H, (cpu.a() & 0x0F) < (val & 0x0F));
                cpu.set_flag_if(Flags::C, cpu.a() < val);

                self.cycles
            }
            SBC(op1, op2) => {
                let carry = if cpu.flag(Flags::C) { 1 } else { 0 };
                let val1 = read_operand_value_u8!(cpu, bus, data, op1);
                let val2 = read_operand_value_u8!(cpu, bus, data, op2);

                let result = val1.wrapping_sub(val2).wrapping_sub(carry);
                cpu.set_a(result);

                cpu.set_flag_if(Flags::Z, result == 0);
                cpu.set_flag(Flags::N);
                cpu.set_flag_if(Flags::H, (val1 & 0x0F) < (val2 & 0x0F) + carry);
                cpu.set_flag_if(Flags::C, (val1 as u16) < (val2 as u16) + (carry as u16));

                self.cycles
            }
            OR(op) => {
                let value = read_operand_value_u8!(cpu, bus, data, op);
                cpu.set_a(cpu.a() | value);

                cpu.set_flag_if(Flags::Z, cpu.a() == 0);
                cpu.clear_flag(Flags::N | Flags::H | Flags::C);

                self.cycles
            }
            DEC(op) => {
                match_size!(
                    op,
                    {
                        let value = read_operand_value_u8!(cpu, bus, data, op);
                        let result = value.wrapping_sub(1);
                        write_to_operand_u8!(cpu, bus, data, op, result);

                        cpu.set_flag_if(Flags::Z, result == 0);
                        cpu.set_flag(Flags::N);
                        cpu.set_flag_if(Flags::H, (value & 0x0F) == 0);
                    },
                    {
                        let value = read_operand_value_u16!(cpu, bus, data, op);
                        let result = value.wrapping_sub(1);
                        write_to_operand_u16!(cpu, bus, op, result);
                    }
                );

                self.cycles
            }
            INC(op) => {
                match_size!(
                    op,
                    {
                        let value = read_operand_value_u8!(cpu, bus, data, op);
                        let result = value.wrapping_add(1);
                        write_to_operand_u8!(cpu, bus, data, op, result);

                        cpu.set_flag_if(Flags::Z, result == 0);
                        cpu.clear_flag(Flags::N);
                        cpu.set_flag_if(Flags::H, (value & 0x0F) == 0xF);
                        cpu.clear_flag(Flags::C);
                    },
                    {
                        let value = read_operand_value_u16!(cpu, bus, data, op);
                        let result = value.wrapping_add(1);
                        write_to_operand_u16!(cpu, bus, op, result);
                    }
                );

                self.cycles
            }
            CP(op) => {
                let value = read_operand_value_u8!(cpu, bus, data, op);
                let result = cpu.a().wrapping_sub(value);
                cpu.set_a(result);

                cpu.set_flag_if(Flags::Z, result == 0);
                cpu.set_flag(Flags::N);
                cpu.set_flag_if(Flags::H, (cpu.a() & 0x0F) < (value & 0x0F));
                cpu.set_flag_if(Flags::C, cpu.a() < value);

                self.size
            }
            CPL => {
                cpu.set_a(0xFF ^ cpu.a());
                cpu.set_flag(Flags::N);
                cpu.set_flag(Flags::H);

                self.cycles
            }

            RLA => {
                let val = cpu.a();
                let result = val << 1 | cpu.flag(Flags::C) as u8;
                cpu.set_a(result);

                cpu.clear_flag(Flags::Z | Flags::N | Flags::H);
                cpu.set_flag_if(Flags::C, val & 0x80 != 0);

                self.cycles
            }
            RRA => {
                let val = cpu.a();
                let result = val >> 1 | (cpu.flag(Flags::C) as u8) << 7;
                cpu.set_a(result);

                cpu.clear_flag(Flags::Z | Flags::N | Flags::H);
                cpu.set_flag_if(Flags::C, val & 0x01 != 0);

                self.cycles
            }
            RLCA => {
                let val = cpu.a();
                let result = val.rotate_left(1);
                cpu.set_a(result);

                cpu.clear_flag(Flags::Z | Flags::N | Flags::H);
                cpu.set_flag_if(Flags::C, val & 0x80 != 0);

                self.cycles
            }
            RRCA => {
                let val = cpu.a();
                let result = val.rotate_right(1);
                cpu.set_a(result);

                cpu.clear_flag(Flags::Z | Flags::N | Flags::H);
                cpu.set_flag_if(Flags::C, val & 0x01 != 0);

                self.cycles
            }

            LD(op1, op2) => {
                match_size!(
                    op1,
                    {
                        let val_u8 = read_operand_value_u8!(cpu, bus, data, op2);
                        write_to_operand_u8!(cpu, bus, data, op1, val_u8);
                    },
                    {
                        let val_u16 = read_operand_value_u16!(cpu, bus, data, op2);
                        write_to_operand_u16!(cpu, bus, op1, val_u16);
                    }
                );

                self.cycles
            }
            LDH(op1, op2) => {
                let val_u8 = read_operand_value_u8!(cpu, bus, data, op2); // A | (n) | (C)
                write_to_operand_u8!(cpu, bus, data, op1, val_u8); // A | (n) | (C)

                self.size
            }

            RST(v) => {
                // push pc on stack
                cpu.set_sp(cpu.sp().wrapping_sub(2));
                bus.write_word(cpu.sp(), cpu.pc());

                // set pc to the address of the rst
                cpu.set_pc(v as u16);

                self.cycles
            }

            DI => {
                cpu.set_ime(false);
                self.cycles
            }
            EI => {
                cpu.set_ime(true);
                self.cycles
            }
            HALT => {
                cpu.halt();
                self.cycles
            }

            CBPrefix => cpu.fetch_cb_instruction(bus).expect("invalid cb prefix"),
            _ => todo!("not implemented: {}", self.operation),
        }
    }

    pub fn execute_cb(&self, cpu: &mut Cpu, bus: &mut impl CpuBus, data: Vec<u8>) -> usize {
        match self.operation {
            SWAP(op) => {
                let val = read_operand_value_u8!(cpu, bus, data, op);
                let result = (val & 0xF0) >> 4 | (val & 0x0F) << 4;
                write_to_operand_u8!(cpu, bus, data, op, result);

                cpu.set_flag_if(Flags::Z, result == 0);
                cpu.clear_flag(Flags::N | Flags::H | Flags::C);

                self.cycles
            }
            BIT(n, op) => {
                let val = read_operand_value_u8!(cpu, bus, data, op);
                let result = val & (1 << n);
                write_to_operand_u8!(cpu, bus, data, op, result);

                cpu.set_flag_if(Flags::Z, result == 0);
                cpu.clear_flag(Flags::N);
                cpu.set_flag(Flags::H);

                self.cycles
            }
            RES(n, op) => {
                let val = read_operand_value_u8!(cpu, bus, data, op);
                let result = val & !(1 << n);
                write_to_operand_u8!(cpu, bus, data, op, result);

                self.cycles
            }
            SET(n, op) => {
                let val = read_operand_value_u8!(cpu, bus, data, op);
                let result = val | (1 << n);
                write_to_operand_u8!(cpu, bus, data, op, result);

                self.cycles
            }
            RLC(op) => {
                let val = read_operand_value_u8!(cpu, bus, data, op);
                let result = val.rotate_left(1);
                write_to_operand_u8!(cpu, bus, data, op, result);

                cpu.set_flag_if(Flags::Z, result == 0);
                cpu.clear_flag(Flags::N | Flags::H);
                cpu.set_flag_if(Flags::C, val & 0x01 != 0);

                self.cycles
            }
            RRC(op) => {
                let val = read_operand_value_u8!(cpu, bus, data, op);
                let result = val.rotate_right(1);
                write_to_operand_u8!(cpu, bus, data, op, result);

                cpu.set_flag_if(Flags::Z, result == 0);
                cpu.clear_flag(Flags::N | Flags::H);
                cpu.set_flag_if(Flags::C, val & 0x80 != 0);

                self.cycles
            }
            RL(op) => {
                let val = read_operand_value_u8!(cpu, bus, data, op);
                let result = val << 1 | cpu.flag(Flags::C) as u8;
                write_to_operand_u8!(cpu, bus, data, op, result);

                cpu.set_flag_if(Flags::Z, result == 0);
                cpu.clear_flag(Flags::N | Flags::H);
                cpu.set_flag_if(Flags::C, val & 0x80 != 0);

                self.cycles
            }
            RR(op) => {
                let val = read_operand_value_u8!(cpu, bus, data, op);
                let result = val >> 1 | (cpu.flag(Flags::C) as u8) << 7;
                write_to_operand_u8!(cpu, bus, data, op, result);

                cpu.set_flag_if(Flags::Z, result == 0);
                cpu.clear_flag(Flags::N | Flags::H);
                cpu.set_flag_if(Flags::C, val & 0x01 != 0);

                self.cycles
            }
            SLA(op) => {
                let val = read_operand_value_u8!(cpu, bus, data, op);
                let result = val << 1;
                write_to_operand_u8!(cpu, bus, data, op, result);

                cpu.set_flag_if(Flags::Z, result == 0);
                cpu.clear_flag(Flags::N | Flags::H);
                cpu.set_flag_if(Flags::C, val & 0x80 != 0);

                self.cycles
            }
            SRA(op) => {
                let val = read_operand_value_u8!(cpu, bus, data, op);
                let result = val >> 1 | (val & 0x01) << 7;
                write_to_operand_u8!(cpu, bus, data, op, result);

                cpu.set_flag_if(Flags::Z, result == 0);
                cpu.clear_flag(Flags::N | Flags::H);
                cpu.set_flag_if(Flags::C, val & 0x01 != 0);

                self.cycles
            }
            SRL(op) => {
                let val = read_operand_value_u8!(cpu, bus, data, op);
                let result = val >> 1;
                write_to_operand_u8!(cpu, bus, data, op, result);

                cpu.set_flag_if(Flags::Z, result == 0);
                cpu.clear_flag(Flags::N | Flags::H);
                cpu.set_flag_if(Flags::C, val & 0x01 != 0);

                self.cycles
            }

            _ => unimplemented!("not implemented: {} (CBPrefix)", self.operation),
        }
    }
}
