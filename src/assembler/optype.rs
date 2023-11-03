#![allow(clippy::unusual_byte_groupings)]

use crate::logging::Log;

use super::{addressing::AddressingList, addressing::AddressingMode, OpSize, value::Value, DataType};

#[derive(Debug)]
pub enum OpType {
    AddSub(bool), //false = add, true = sub
    AddSubA(bool),
    AddSubQ(bool),
    AddSubX(bool),
    Bcd(bool), //false = ABCD, true = SBCD
    BitManip(u8),
    Bkpt,
    Branch(u8),
    Chk,
    Cmp,
    Cmpa,
    Cmpm,
    Dbcc(u8),
    Eor,
    Exg,
    Ext,
    Immediates(u8),
    Jump(bool), //false = jsr, true = jmp
    Lea,
    Link,
    Misc1(u8),
    Move,
    MoveA,
    Movem,
    Movep,
    MoveQ,
    MulDiv(u16),
    Nbcd,
    NoOperands(u16),
    OrAnd(bool), //false = or, true = and
    Pea,
    Rotation(u8, bool),
    Rtd,
    Scc(u8),
    Stop,
    Swap,
    Tas,
    Trap,
    Tst,
    Unlk,

    Data(DataType, Vec<Value>),
}

impl OpType {
    pub fn format(&self) -> u16 {
        use OpType::*;

        match self {
            AddSub(add_sub) | AddSubA(add_sub) | AddSubX(add_sub) => match add_sub {
                false => 0b1101 << 12,
                true  => 0b1001 << 12,
            },

            AddSubQ(add_sub) => match add_sub {
                false => 0b0101_0000 << 8,
                true  => 0b0101_0001 << 8,
            },

            Bcd(add_sub) => match add_sub {
                false => 0b1100_000_10000 << 4,
                true  => 0b1000_000_10000 << 4,
            },

            BitManip(format) => (*format as u16) << 6,
            Bkpt => 0b0100100001001 << 3,
            Branch(cond) => u16::from_be_bytes([(0b0110 << 4) | cond, 0]),
            Chk  => 0b0100_000_110 << 6,
            Cmp  => 0b1011 << 12,
            Cmpa => 0b1011_000_0_11 << 6,
            Cmpm => 0b1011_000_1_00_001 << 3,
            Dbcc(cond) => (0b0101_0000_11_001 << 3) | ((*cond as u16) << 8),
            Eor => 0b1011_000_1 << 8,
            Exg  => 0b1100_000_1 << 8,
            Ext  => 0b0100_100_0_1 << 7,
            Immediates(imm) => (*imm as u16) << 9,

            Jump(jxx) => match jxx {
                false => 0b0100111010 << 6,
                true  => 0b0100111011 << 6,
            },

            Lea  => 0b0100_000_111 << 6,
            Link => 0b0100_111001010 << 3,
            Misc1(format) => (*format as u16) << 8,
            Move => 0,
            MoveA => 0b001 << 6,
            Movem => 0b0100_1_0_001 << 7,
            Movep => 0b1_0_0_001 << 3,
            MoveQ => 0b0111 << 12,
            MulDiv(format) => *format,
            Nbcd => 0b0100_100_000 << 6,
            NoOperands(data) => *data,

            OrAnd(format) => match format {
                false => 0b1000 << 12,
                true  => 0b1100 << 12,
            },

            Pea  => 0b0100_100_001 << 6,
            Rotation(_, _) => 0b1110 << 12,
            Rtd => 0b0100111001110100,
            Scc(cond) => (0b0101_0000_11 << 6) | ((*cond as u16) << 8),
            Stop => 0b0100_111001110010,
            Swap => 0b0100_100_001_000 << 3,
            Tas  => 0b0100_101011 << 6,
            Trap => 0b0100_11100100 << 4,
            Tst  => 0b0100_1010 << 8,
            Unlk => 0b0100_111001011 << 3,

            Data(_, _) => 0, //unused
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

            "bkpt" => Bkpt,
            "rtd" => Rtd,

            _ => return Err(Log::InvalidOp),
        })
    }

    pub fn valid_size(&self, size: OpSize) -> Result<(), Log> {
        use OpSize::*;
        use OpType::*;

        let valid = match self {
            Chk | Link | MulDiv(_) => W,

            Branch(_) => BW,
            BitManip(_) => BL,
            AddSubA(_) | Cmpa | Ext | MoveA | Movem | Movep => WL,

            AddSub(_) | AddSubQ(_) | AddSubX(_) | Cmp | Cmpm | Eor | Immediates(_) |
            Misc1(_) | Move | OrAnd(_) | Rotation(_, _) | Tst => BWL,

            Jump(_) | NoOperands(_) | Stop | Trap | Unlk | Bkpt | Rtd => Unsized,

            Bcd(_) | Nbcd | Scc(_) | Tas => BU,
            Dbcc(_) | Swap => WU,
            Exg | Lea | MoveQ | Pea => LU,

            Data(_, _) => Unsized, //unused
        };

        match size.mask() & valid.mask() != 0 {
            true  => Ok(()),
            false => Err(Log::UnsupportedSuffix),
        }
    }

    fn mode_lists(&self, modes: &[AddressingMode; 2]) -> [Option<AddressingList>; 2] {
        use AddressingList::*;
        use OpType::*;

        match self {
            Branch(_) | Rtd                => [Some(Displacement), None],
            NoOperands(_)                  => [None, None],
            AddSubA(_) | MoveA | Cmpa      => [Some(All), Some(AddressRegister)],
            AddSubQ(_)                     => [Some(DataQuick), Some(Alterable)],
            Jump(_) | Pea                  => [Some(Control), None],
            MoveQ                          => [Some(DataQuick), Some(DataRegister)],
            Lea                            => [Some(Control), Some(AddressRegister)],
            Chk                            => [Some(DataAddressing), Some(DataRegister)],
            Tst                            => [Some(All), None],
            Ext | Swap                     => [Some(DataRegister), None],
            Unlk                           => [Some(AddressRegister), None],
            Link                           => [Some(AddressRegister), Some(Immediate)],
            Trap | Bkpt                    => [Some(DataQuick), None],
            Stop                           => [Some(Immediate), None],
            Cmp                            => [Some(All), Some(DataRegister)],
            Cmpm                           => [Some(AddressPostincrement), Some(AddressPostincrement)],
            MulDiv(_)                      => [Some(DataAlterable), Some(DataRegister)],
            Eor                            => [Some(DataRegister), Some(DataAlterable)],
            Dbcc(_)                        => [Some(DataRegister), Some(Displacement)],
            Misc1(_) | Tas | Scc(_) | Nbcd => [Some(DataAlterable), None],

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
                    AddressingMode::CCR => [Some(CCR), Some(DataAlterable)],
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
                let valid_ccr_sr_modes = [0b000, 0b001, 0b101]; //ori, andi, eori

                if valid_ccr_sr_modes.contains(imm) {
                    match modes[1] {
                        AddressingMode::CCR => [Some(Immediate), Some(CCR)],
                        AddressingMode::SR => [Some(Immediate), Some(SR)],
                        _ => [Some(Immediate), Some(DataAlterable)],
                    }
                } else {
                    [Some(Immediate), Some(DataAlterable)]
                }
            }

            Data(_, _) => [None, None], //unused
        }
    }

    pub fn is_valid_modes(&self, modes: &[AddressingMode; 2]) -> Result<(), Log> {
        for (valid_list, mode) in self.mode_lists(modes).iter().zip(modes) {
            if let Some(valid_list) = valid_list {
                if !valid_list.contains(mode) {
                    return Err(Log::InvalidAddressingMode);
                }
            } else if *mode != AddressingMode::Empty {
                return Err(Log::TooManyOperands);
            }
        }

        Ok(())
    }
}
