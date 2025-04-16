use crate::cpu::addressing_mode::{Condition, Op, Reg, CC};
use crate::cpu::instruction::Operation::*;
use crate::cpu::{CpuBus, Flags, CPU};
use crate::z;
use bitflags::Flag;
use log::debug;
use std::convert;

macro_rules! read_u16_le {
    ($data:expr) => {
        ($data[1] as u16) << 8 | ($data[0] as u16)
    };
    ($data:expr, $index:expr) => {
        ($data[$index + 1] as u16) << 8 | ($data[$index] as u16)
    };
}

macro_rules! read_operand_value {
    ($cpu:expr, $bus:expr, $data:expr, $op:expr) => {
        match $op {
            z!("A") => $cpu.a,
            z!("B") => $cpu.b,
            z!("C") => $cpu.c,
            z!("D") => $cpu.d,
            z!("E") => $cpu.e,
            z!("H") => $cpu.h,
            z!("L") => $cpu.l,
            z!("n") => $data[0],
            z!("(HL)") => $bus.read_byte((($cpu.h as u16) << 8) | ($cpu.l as u16)),
            z!("(nn)") => $bus.read_byte(read_u16_le!($data)),
            _ => {
                debug!("Opérande non supporté: {:?}", $op);
                unreachable!("Opérande non supporté")
            }
        }
    };
}
macro_rules! read_operand_value_16u {
    ($cpu:expr, $bus:expr, $data:expr, $op:expr) => {
        match $op {
            z!("nn") => read_u16_le!($data),
            z!("HL") => ($cpu.h as u16) << 8 | ($cpu.l as u16),
            _ => {
                debug!("Opérande non supporté: {:?}", $op);
                unreachable!("Opérande non supporté")
            }
        }
    };
}
macro_rules! write_to_operand {
    ($cpu:expr, $bus:expr, $op:expr, $value:expr) => {
        match $op {
            z!("A") => $cpu.a = $value,
            z!("B") => $cpu.b = $value,
            z!("C") => $cpu.c = $value,
            z!("D") => $cpu.d = $value,
            z!("E") => $cpu.e = $value,
            z!("H") => $cpu.h = $value,
            z!("L") => $cpu.l = $value,
            z!("AF") => {
                $cpu.a = (($value as u16) >> 8) as u8;
                $cpu.f = Flags::from_bits_truncate(($value as u16) as u8);
            }
            z!("BC") => {
                $cpu.b = (($value as u16) >> 8) as u8;
                $cpu.c = ($value as u16) as u8;
            }
            z!("DE") => {
                $cpu.d = (($value as u16) >> 8) as u8;
                $cpu.e = ($value as u16) as u8;
            }
            z!("HL") => {
                $cpu.h = (($value as u16) >> 8) as u8;
                $cpu.l = ($value as u16) as u8;
            }
            z!("(HL)") => {
                let addr = (($cpu.h as u16) << 8) | ($cpu.l as u16);
                $bus.write_byte(addr, $value);
            }
            z!("(HL+)") => {
                let addr = (($cpu.h as u16) << 8) | ($cpu.l as u16);
                $bus.write_byte(addr, $value);

                let hl = addr.wrapping_add(1);
                $cpu.h = (hl >> 8) as u8;
                $cpu.l = hl as u8;
            }
            z!("(HL-)") => {
                let addr = (($cpu.h as u16) << 8) | ($cpu.l as u16);
                $bus.write_byte(addr, $value);

                let hl = addr.wrapping_sub(1);
                $cpu.h = (hl >> 8) as u8;
                $cpu.l = hl as u8;
            }
            _ => unreachable!("Opérande d'écriture non supporté: {:?}", $op),
        }
    };
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
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

    pub fn execute(&self, cpu: &mut CPU, bus: &mut impl CpuBus, data: Vec<u8>) -> usize {
        match self.operation {
            NOP => self.cycles,

            JP(op) if matches!(op, z!("HL") | z!("nn")) => {
                let address = read_operand_value_16u!(cpu, bus, data, op);
                debug!("jump to ${:04x}", address);
                cpu.pc = address;

                self.cycles
            }
            JPcc(cc, op) => {
                if cpu.check_condition(cc) {
                    let address = read_operand_value_16u!(cpu, bus, data, op);
                    debug!("jump to ${:04x}", address);
                    cpu.pc = address;

                    self.cycles
                } else {
                    self.cycles_not_taken
                }
            }

            AND(op) => {
                let value = read_operand_value!(cpu, bus, data, op);
                cpu.a &= value;

                cpu.set_flag_if(Flags::Z, cpu.a == 0);
                cpu.clear_flag(Flags::N);
                cpu.set_flag(Flags::H);
                cpu.clear_flag(Flags::C);

                self.cycles
            }
            XOR(op) => {
                let value = read_operand_value!(cpu, bus, data, op);
                cpu.a ^= value;

                cpu.set_flag_if(Flags::Z, cpu.a == 0);
                cpu.clear_flag(Flags::N | Flags::H | Flags::C);

                self.cycles
            }
            ADD(z!("A"), op2) => {
                let value = read_operand_value!(cpu, bus, data, op2);

                let result = cpu.a.wrapping_add(value);
                cpu.set_flag_if(Flags::Z, result == 0);
                cpu.clear_flag(Flags::N);
                cpu.set_flag_if(Flags::H, (cpu.a & 0x0F) + (value & 0x0F) > 0x0F);
                cpu.set_flag_if(Flags::C, cpu.a as u16 + value as u16 > 0xFF);
                cpu.a = result;

                self.cycles
            }
            ADC(z!("A"), op2) => {
                let carry = if cpu.get_flag(Flags::C) { 1 } else { 0 };
                let val2 = cpu.a;
                let val1 = read_operand_value!(cpu, bus, data, op2);

                let result = val1.wrapping_add(val2).wrapping_add(carry);
                cpu.a = result;

                cpu.set_flag_if(Flags::Z, result == 0);
                cpu.clear_flag(Flags::N);
                cpu.set_flag_if(Flags::H, (val1 & 0x0F) + (val2 & 0x0F) + carry > 0x0F);
                cpu.set_flag_if(Flags::C, val1 as u16 + val2 as u16 + carry as u16 > 0xFF);

                self.cycles
            }
            SUB(op) => {
                let val = read_operand_value!(cpu, bus, data, op);

                let result = cpu.a.wrapping_sub(val);
                cpu.a = result;

                cpu.set_flag_if(Flags::Z, result == 0);
                cpu.set_flag(Flags::N);
                cpu.set_flag_if(Flags::H, (cpu.a & 0x0F) < (val & 0x0F));
                cpu.set_flag_if(Flags::C, cpu.a < val);

                self.cycles
            }
            SBC(op1, op2) => {
                let carry = if cpu.get_flag(Flags::C) { 1 } else { 0 };
                let val1 = read_operand_value!(cpu, bus, data, op1);
                let val2 = read_operand_value!(cpu, bus, data, op2);

                let result = val1.wrapping_sub(val2).wrapping_sub(carry);
                cpu.a = result;

                cpu.set_flag_if(Flags::Z, result == 0);
                cpu.set_flag(Flags::N);
                cpu.set_flag_if(Flags::H, (val1 & 0x0F) < (val2 & 0x0F) + carry);
                cpu.set_flag_if(Flags::C, (val1 as u16) < (val2 as u16) + (carry as u16));

                self.cycles
            }
            OR(op) => {
                cpu.a |= read_operand_value!(cpu, bus, data, op);

                cpu.set_flag_if(Flags::Z, cpu.a == 0);
                cpu.clear_flag(Flags::N | Flags::H | Flags::C);

                self.cycles
            }
            DEC(op) => {
                let mut value = read_operand_value!(cpu, bus, data, op);
                value = value.wrapping_sub(1);
                write_to_operand!(cpu, bus, op, value);

                cpu.set_flag_if(Flags::Z, value == 0);
                cpu.set_flag(Flags::N);
                cpu.set_flag_if(Flags::H, value & 0x0F == 0);
                cpu.clear_flag(Flags::C);

                self.cycles
            }

            LD(op1, op2) => {
                let op2_value = read_operand_value!(cpu, bus, data, op2);
                write_to_operand!(cpu, bus, op1, op2_value);

                self.cycles
            }

            _ => todo!("not implemented: {}", self.operation),
        }
    }
}
