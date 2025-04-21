use crate::cpu::addressing_mode::Reg;
use crate::cpu::addressing_mode::{CC, Op};
use crate::cpu::instruction::Operation::*;
use crate::cpu::{Cpu, CpuBus, Flags};
use crate::z;
use log::{debug, trace};

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
            z!("(nn)") => $bus.read_byte(read_u16_le!($data)),
            _ => {
                debug!("Unsupported operand: {:?}", $op);
                unreachable!("Unsupported operand")
            }
        }
    };
}
macro_rules! read_operand_value_u16 {
    ($cpu:expr, $bus:expr, $data:expr, $op:expr) => {
        match $op {
            z!("nn") => read_u16_le!($data),
            z!("HL") => $cpu.hl(),
            _ => {
                debug!("Unsupported operand: {:?}", $op);
                unreachable!("Unsupported operand")
            }
        }
    };
}
macro_rules! write_to_operand_u8 {
    ($cpu:expr, $bus:expr, $op:expr, $value:expr) => {
        match $op {
            z!("A") => $cpu.set_a($value),
            z!("B") => $cpu.set_b($value),
            z!("C") => $cpu.set_c($value),
            z!("D") => $cpu.set_d($value),
            z!("E") => $cpu.set_e($value),
            z!("H") => $cpu.set_h($value),
            z!("L") => $cpu.set_l($value),
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
                debug!("Unsupported operand: {:?}", $op);
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
            _ => {
                debug!("Unsupported operand: {:?}", $op);
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
            ADD(z!("A"), op2) => {
                let value = read_operand_value_u8!(cpu, bus, data, op2);

                let result = cpu.a().wrapping_add(value);
                cpu.set_flag_if(Flags::Z, result == 0);
                cpu.clear_flag(Flags::N);
                cpu.set_flag_if(Flags::H, (cpu.a() & 0x0F) + (value & 0x0F) > 0x0F);
                cpu.set_flag_if(Flags::C, cpu.a() as u16 + value as u16 > 0xFF);
                cpu.set_a(result);

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
                cpu.set_a(cpu.a() | read_operand_value_u8!(cpu, bus, data, op));

                cpu.set_flag_if(Flags::Z, cpu.a() == 0);
                cpu.clear_flag(Flags::N | Flags::H | Flags::C);

                self.cycles
            }
            DEC(op) => {
                let mut value = read_operand_value_u8!(cpu, bus, data, op);
                value = value.wrapping_sub(1);
                //write_to_operand!(cpu, bus, op, value);

                cpu.set_flag_if(Flags::Z, value == 0);
                cpu.set_flag(Flags::N);
                cpu.set_flag_if(Flags::H, value & 0x0F == 0);
                cpu.clear_flag(Flags::C);

                self.cycles
            }

            LD(op1, op2) => {
                match_size!(
                    op1,
                    {
                        let val_u8 = read_operand_value_u8!(cpu, bus, data, op2);
                        write_to_operand_u8!(cpu, bus, op1, val_u8);
                    },
                    {
                        let val_u16 = read_operand_value_u16!(cpu, bus, data, op2);
                        write_to_operand_u16!(cpu, bus, op1, val_u16);
                    }
                );

                self.cycles
            }

            RST(v) => {
                // push pc on stack
                cpu.set_sp(cpu.sp().wrapping_sub(2));
                bus.write_word(cpu.sp(), cpu.pc());

                // set pc to the address of the rst
                cpu.set_pc(v as u16);

                self.cycles
            }

            _ => todo!("not implemented: {}", self.operation),
        }
    }
}
