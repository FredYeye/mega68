#![allow(clippy::unusual_byte_groupings)]

use crate::Log;

use super::{addressing::AddressingList, addressing::AddressingMode, OpSize};

#[derive(Debug)]
pub enum OpType {
    Branch(u8),
    NoOperands(u16),
    AddSub(bool), //false = add, true = sub
    AddSubA(bool),
    AddSubX(bool),
    AddSubQ(bool),
    Immediates(u8),
    Jump(bool), //false = jsr, true = jmp
    Move,
    MoveA,
    Bcd(bool), //false = ABCD, true = SBCD
    BitManip(u8),
    Misc1(u8),
    OrAnd(bool), //false = or, true = and
    MoveQ,
    Rotation(u8, bool),
    Lea,
    Chk,
    Exg,
    Tst,
    Ext,
    Swap,
    Unlk,
    Link,
    Trap,
    Tas,
    Stop,
    Pea,
    Cmp,
    Cmpa,
    Cmpm,
    Scc(u8),
    Nbcd,
    MulDiv(u16),
    Eor,
    Dbcc(u8),
    Movep,
    Movem,

    Data(Vec<u16>),
}

impl OpType {
    pub fn format(&self) -> u16 {
        use OpType::*;

        match self {
            Branch(cond) => u16::from_be_bytes([(0b0110 << 4) | cond, 0]),
            NoOperands(data) => *data,
            Immediates(imm) => (*imm as u16) << 9,

            AddSub(add_sub) | AddSubA(add_sub) | AddSubX(add_sub) => match add_sub {
                false => 0b1101 << 12,
                true  => 0b1001 << 12,
            },

            AddSubQ(add_sub) => match add_sub {
                false => 0b0101_0000 << 8,
                true  => 0b0101_0001 << 8,
            },

            Jump(jxx) => match jxx {
                false => 0b0100111010 << 6,
                true  => 0b0100111011 << 6,
            },

            Move => 0,
            MoveA => 0b001 << 6,

            Bcd(add_sub) => match add_sub {
                false => 0b1100_000_10000 << 4,
                true  => 0b1000_000_10000 << 4,
            },

            BitManip(format) => (*format as u16) << 6,

            Misc1(format) => (*format as u16) << 8,

            OrAnd(format) => match format {
                false => 0b1000 << 12,
                true  => 0b1100 << 12,
            },

            MoveQ => 0b0111 << 12,

            Rotation(_, _) => 0b1110 << 12,

            Lea  => 0b0100_000_111 << 6,
            Chk  => 0b0100_000_110 << 6,
            Exg  => 0b1100_000_1 << 8,
            Tst  => 0b0100_1010 << 8,
            Ext  => 0b0100_100_0_1 << 7,
            Swap => 0b0100_100_001_000 << 3,
            Unlk => 0b0100_111001011 << 3,
            Link => 0b0100_111001010 << 3,
            Trap => 0b0100_11100100 << 4,
            Tas  => 0b0100_101011 << 6,
            Stop => 0b0100_111001110010,
            Pea  => 0b0100_100_001 << 6,

            Cmp  => 0b1011 << 12,
            Cmpa => 0b1011_000_0_11 << 6,
            Cmpm => 0b1011_000_1_00_001 << 3,

            Scc(cond) => (0b0101_0000_11 << 6) | ((*cond as u16) << 8),
            Nbcd => 0b0100_100_000 << 6,
            MulDiv(format) => *format,
            Eor => 0b1011_000_1 << 8,
            Dbcc(cond) => (0b0101_0000_11_001 << 3) | ((*cond as u16) << 8),
            Movep => 0b1_0_0_001 << 3,
            Movem => 0b0100_1_0_001 << 7,

            Data(_) => 0, //unused
        }
    }

    pub fn parse_op(op: &str) -> Result<Self, Log> {
        use OpType::*;

        Ok(match op.to_lowercase().as_str() {
            "bra" => Branch(0b0000),
            "bsr" => Branch(0b0001),
            "bhi" => Branch(0b0010),
            "bls" => Branch(0b0011),
            "bcc" => Branch(0b0100),
            "bcs" => Branch(0b0101),
            "bne" => Branch(0b0110),
            "beq" => Branch(0b0111),
            "bvc" => Branch(0b1000),
            "bvs" => Branch(0b1001),
            "bpl" => Branch(0b1010),
            "bmi" => Branch(0b1011),
            "bge" => Branch(0b1100),
            "blt" => Branch(0b1101),
            "bgt" => Branch(0b1110),
            "ble" => Branch(0b1111),

            "dbt"  => Dbcc(0b0000),
            "dbf"  => Dbcc(0b0001),
            "dbhi" => Dbcc(0b0010),
            "dbls" => Dbcc(0b0011),
            "dbcc" => Dbcc(0b0100),
            "dbcs" => Dbcc(0b0101),
            "dbne" => Dbcc(0b0110),
            "dbeq" => Dbcc(0b0111),
            "dbvc" => Dbcc(0b1000),
            "dbvs" => Dbcc(0b1001),
            "dbpl" => Dbcc(0b1010),
            "dbmi" => Dbcc(0b1011),
            "dbge" => Dbcc(0b1100),
            "dblt" => Dbcc(0b1101),
            "dbgt" => Dbcc(0b1110),
            "dble" => Dbcc(0b1111),

            "st"  => Scc(0b0000),
            "sf"  => Scc(0b0001),
            "shi" => Scc(0b0010),
            "sls" => Scc(0b0011),
            "scc" => Scc(0b0100),
            "scs" => Scc(0b0101),
            "sne" => Scc(0b0110),
            "seq" => Scc(0b0111),
            "svc" => Scc(0b1000),
            "svs" => Scc(0b1001),
            "spl" => Scc(0b1010),
            "smi" => Scc(0b1011),
            "sge" => Scc(0b1100),
            "slt" => Scc(0b1101),
            "sgt" => Scc(0b1110),
            "sle" => Scc(0b1111),

            "illegal" => NoOperands(0b0100_101011111100),
            "nop"     => NoOperands(0b0100_111001110001),
            "reset"   => NoOperands(0b0100_111001110000),
            "rte"     => NoOperands(0b0100_111001110011),
            "rtr"     => NoOperands(0b0100_111001110111),
            "rts"     => NoOperands(0b0100_111001110101),
            "trapv"   => NoOperands(0b0100_111001110110),

            "add" => AddSub(false),
            "sub" => AddSub(true),

            "adda" => AddSubA(false),
            "suba" => AddSubA(true),

            "addx" => AddSubX(false),
            "subx" => AddSubX(true),

            "addq" => AddSubQ(false),
            "subq" => AddSubQ(true),

            "ori"  => Immediates(0b000),
            "andi" => Immediates(0b001),
            "subi" => Immediates(0b010),
            "addi" => Immediates(0b011),
            "eori" => Immediates(0b101),
            "cmpi" => Immediates(0b110),

            "jmp" => Jump(false),
            "jsr" => Jump(true),

            "move" => Move,
            "movea" => MoveA,

            "abcd" => Bcd(false),
            "sbcd" => Bcd(true),

            "btst" => BitManip(0b00),
            "bchg" => BitManip(0b01),
            "bclr" => BitManip(0b10),
            "bset" => BitManip(0b11),

            "negx" => Misc1(0b0100_0000),
            "clr"  => Misc1(0b0100_0010),
            "neg"  => Misc1(0b0100_0100),
            "not"  => Misc1(0b0100_0110),

            "or"  => OrAnd(false),
            "and" => OrAnd(true),

            "moveq" => MoveQ,

            "asl"  => Rotation(0b00, true),
            "asr"  => Rotation(0b00, false),
            "lsl"  => Rotation(0b01, true),
            "lsr"  => Rotation(0b01, false),
            "roxl" => Rotation(0b10, true),
            "roxr" => Rotation(0b10, false),
            "rol"  => Rotation(0b11, true),
            "ror"  => Rotation(0b11, false),

            "lea" => Lea,
            "chk" => Chk,
            "exg" => Exg,
            "tst" => Tst,
            "ext" => Ext,
            "swap" => Swap,
            "unlk" => Unlk,
            "link" => Link,
            "trap" => Trap,
            "tas" => Tas,
            "stop" => Stop,
            "pea" => Pea,

            "cmp"  => Cmp,
            "cmpa" => Cmpa,
            "cmpm" => Cmpm,

            "nbcd" => Nbcd,

            "divu" => MulDiv(0b1000_000_011 << 6),
            "divs" => MulDiv(0b1000_000_111 << 6),
            "mulu" => MulDiv(0b1100_000_011 << 6),
            "muls" => MulDiv(0b1100_000_111 << 6),

            "eor" => Eor,

            "movep" => Movep,
            "movem" => Movem,

            _ => return Err(Log::InvalidOp),
        })
    }

    pub fn valid_size(&self, size: OpSize) -> Result<(), Log> {
        use OpSize::*;
        use OpType::*;

        let valid = match self {
            Bcd(_) | Nbcd | Scc(_) | Tas => B,
            Chk | Dbcc(_) | Link | MulDiv(_) => W,
            Exg | Lea | Pea => L,

            Branch(_) => BW,
            BitManip(_) => BL,
            AddSubA(_) | Cmpa | Ext | MoveA | Movem | Movep => WL,

            AddSub(_) | AddSubQ(_) | AddSubX(_) | Cmp | Cmpm | Eor | Immediates(_) |
            Misc1(_) | Move | OrAnd(_) | Rotation(_, _) | Tst => BWL,

            Jump(_) | NoOperands(_) | Stop | Trap | Unlk => Unsized,

            Swap => WU,
            MoveQ => LU,

            Data(_) => Unsized, //unused
        };

        match size.mask() & valid.mask() != 0 {
            true  => Ok(()),
            false => Err(Log::UnsupportedSuffix),
        }
    }

    pub fn is_valid_modes(&self, modes: &[AddressingMode; 2]) {
        use AddressingList::*;
        use OpType::*;

        let valid_addr_lists = match self {
            Branch(_)                                 => [Some(Displacement), None],
            NoOperands(_)                             => [None, None],
            AddSubA(_) | OpType::MoveA | OpType::Cmpa => [Some(All), Some(AddressRegister)],
            AddSubQ(_)                                => [Some(DataQuick), Some(Alterable)],
            Jump(_) | OpType::Pea                     => [Some(Control), None],
            MoveQ                                     => [Some(DataQuick), Some(DataRegister)],
            Lea                                       => [Some(Control), Some(AddressRegister)],
            Chk                                       => [Some(DataAddressing), Some(DataRegister)],
            Tst                                       => [Some(All), None],
            Ext | OpType::Swap                        => [Some(DataRegister), None],
            Unlk                                      => [Some(AddressRegister), None],
            Link                                      => [Some(AddressRegister), Some(Immediate)],
            Trap                                      => [Some(DataQuick), None],
            Stop                                      => [Some(Immediate), None],
            Cmp                                       => [Some(All), Some(DataRegister)],
            Cmpm                                      => [Some(AddressPostincrement), Some(AddressPostincrement)],
            MulDiv(_)                                 => [Some(DataAlterable), Some(DataRegister)],
            Eor                                       => [Some(DataRegister), Some(DataAlterable)],
            Dbcc(_)                                   => [Some(DataRegister), Some(Displacement)],
            Misc1(_) | Tas | Scc(_) | Nbcd            => [Some(DataAlterable), None],

            AddSub(_) => match modes[1] {
                AddressingMode::DataRegister(_) => [Some(All), Some(DataRegister)],
                _ => [Some(DataRegister), Some(MemoryAlterable)],
            },

            AddSubX(_) | Bcd(_) => match modes[0] {
                AddressingMode::DataRegister(_) => [Some(DataRegister), Some(DataRegister)],
                _ => [Some(AddressPredecrement), Some(AddressPredecrement)],
            },

            BitManip(_) => match modes[0] {
                AddressingMode::DataRegister(_) => [Some(DataRegister), Some(DataAddressing)],
                _ => [Some(Immediate), Some(DataAddressing2)],
            },

            OrAnd(_) => match modes[1] {
                AddressingMode::DataRegister(_) => [Some(DataAddressing), Some(DataRegister)],
                _ => [Some(DataRegister), Some(MemoryAlterable)],
            },

            Rotation(_, _) => match modes[1] {
                AddressingMode::DataRegister(_) => [Some(DataRegisterDataQuick), Some(DataRegister)],
                _ => [Some(MemoryAlterable), None],
            },

            Exg => match modes[1] {
                AddressingMode::DataRegister(_) => [Some(DataRegister), Some(DataRegister)],
                _ => [Some(Register), Some(AddressRegister)],
            },

            Movep => match modes[0] {
                AddressingMode::DataRegister(_) => [Some(DataRegister), Some(AddressDisplacement)],
                _ => [Some(AddressDisplacement), Some(DataRegister)],
            },

            Movem => match modes[0] {
                AddressingMode::RegisterList(_) => [Some(RegisterList), Some(MovemDst)],
                _ => [Some(MovemSrc), Some(RegisterList)],
            },

            Move => {
                match modes[0] {
                    AddressingMode::SR => [Some(SR), Some(DataAlterable)],
                    AddressingMode::USP => [Some(USP), Some(DataRegister)],
                    _ => {
                        match modes[1] {
                            AddressingMode::CCR => [Some(DataAddressing), Some(CCR)],
                            AddressingMode::SR => [Some(DataAddressing), Some(SR)],
                            AddressingMode::USP => [Some(DataRegister), Some(USP)],
                            _ => [Some(All), Some(DataAlterable)], //normal move
                        }
                    }
                }
            }

            Immediates(imm) => {
                let valid_ccr_sr_modes = [0b000, 0b001, 0b101];

                match modes[1] {
                    AddressingMode::CCR =>  match valid_ccr_sr_modes.contains(imm) {
                        true  => [Some(Immediate), Some(CCR)],
                        false => todo!("error"),
                    }

                    AddressingMode::SR =>  match valid_ccr_sr_modes.contains(imm) {
                        true  => [Some(Immediate), Some(SR)],
                        false => todo!("error"),
                    }

                    _ => [Some(Immediate), Some(DataAlterable)],
                }
            }

            Data(_) => [None, None], //unused
        };

        for (valid_list, mode) in valid_addr_lists.iter().zip(modes) {
            if let Some(valid_list2) = valid_list {
                if !valid_list2.check_mode(mode) {
                    todo!("invalid addressing mode");
                }
            } else if *mode != AddressingMode::Empty {
                todo!("invalid addressing mode (too many operands)");
            }
        }
    }
}
