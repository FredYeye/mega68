use crate::Log;

use super::{parse_n, OpSize, OpType, Value, CCR_MASK, MOVEM_MASK, SR_MASK, USP_MASK};

#[derive(Debug, PartialEq)]
pub enum AddressingMode {
    DataRegister(u8),
    AddressRegister(u8),
    Address(u8),
    AddressPostincrement(u8),
    AddressPredecrement(u8),
    AddressDisplacement(i16, u8),
    AddressIndex(i8, u8, u8, bool, bool),
    PCDisplacement(i16),
    PCIndex(i8, u8, bool, bool), // disp, Xn, reg type, reg size
    AbsoluteShort(u16),
    AbsoluteLong(Value),
    Immediate(OpSize, u32),

    BranchDisplacement(OpSize, Value),
    RegisterList(u16),
    DataQuick(u8),

    CCR,
    SR,
    USP,

    Empty,
}

impl AddressingMode {
    pub fn ea_size(modes: &[Self; 2]) -> u8 {
        let mut count = 0;

        for mode in modes {
            count += match mode {
                Self::AddressDisplacement(_, _)
                | Self::AddressIndex(_, _, _, _, _)
                | Self::PCDisplacement(_)
                | Self::PCIndex(_, _, _, _)
                | Self::AbsoluteShort(_)
                | Self::RegisterList(_) => 2,

                Self::AbsoluteLong(_) => 4,

                Self::Immediate(size, _) => match size {
                    OpSize::L => 4,
                    _ => 2,
                },

                Self::BranchDisplacement(size, _) => match size {
                    OpSize::W => 2,
                    _ => 0,
                },

                _ => 0,
            };
        }

        count
    }

    pub fn mask_bit(&self) -> u32 {
        match self {
            Self::DataRegister(_)             => 0b0_000_000_000000000001,
            Self::AddressRegister(_)          => 0b0_000_000_000000000010,
            Self::Address(_)                  => 0b0_000_000_000000000100,
            Self::AddressPostincrement(_)     => 0b0_000_000_000000001000,
            Self::AddressPredecrement(_)      => 0b0_000_000_000000010000,
            Self::AddressDisplacement(_, _)   => 0b0_000_000_000000100000,
            Self::AddressIndex(_, _, _, _, _) => 0b0_000_000_000001000000,
            Self::PCDisplacement(_)           => 0b0_000_000_000010000000,
            Self::PCIndex(_, _, _, _)         => 0b0_000_000_000100000000,
            Self::AbsoluteShort(_)            => 0b0_000_000_001000000000,
            Self::AbsoluteLong(_)             => 0b0_000_000_010000000000,
            Self::Immediate(_, _)             => 0b0_000_000_100000000000,
            Self::BranchDisplacement(_, _)    => 0b0_000_001_000000000000,
            Self::RegisterList(_)             => 0b0_000_010_000000000000,
            Self::DataQuick(_)                => 0b0_000_100_000000000000,
            Self::CCR                         => 0b0_001_000_000000000000,
            Self::SR                          => 0b0_010_000_000000000000,
            Self::USP                         => 0b0_100_000_000000000000,
            Self::Empty                       => 0b0_000_000_000000000000,
        }
    }

    pub fn effective_addressing(&self, labels: &std::collections::HashMap<String, u32>, defines: &std::collections::HashMap<String, u32>) -> Result<(u16, Vec<u16>), Log> {
        let ea = match self {
            AddressingMode::DataRegister(reg) => (0b000, *reg, vec![]),
            AddressingMode::AddressRegister(reg) => (0b001, *reg, vec![]),

            AddressingMode::Address(reg) => (0b010, *reg, vec![]),
            AddressingMode::AddressPostincrement(reg) => (0b011, *reg, vec![]),
            AddressingMode::AddressPredecrement(reg) => (0b100, *reg, vec![]),
            AddressingMode::AddressDisplacement(disp, reg) => (0b101, *reg, vec![*disp as u16]),

            AddressingMode::AddressIndex(disp, reg_a, reg, reg_type, reg_size) => (
                0b110,
                *reg_a,
                vec![((*reg_type as u16) << 15) | ((*reg as u16) << 12) | ((*reg_size as u16) << 11) | *disp as u16],
            ),

            AddressingMode::PCDisplacement(disp) => (0b111, 0b010, vec![*disp as u16]),
            AddressingMode::PCIndex(disp, reg, reg_type, reg_size) => (
                0b111,
                0b011,
                vec![((*reg_type as u16) << 15) | ((*reg as u16) << 12) | ((*reg_size as u16) << 11) | *disp as u16],
            ),

            AddressingMode::AbsoluteShort(addr) => (0b111, 0b000, vec![*addr]),
            AddressingMode::AbsoluteLong(value) => {
                let addr = value.resolve_value(labels, defines)?;
                (0b111, 0b001, vec![(addr >> 16) as u16, addr as u16])
            }

            AddressingMode::Immediate(size, value) => {
                //todo: warn if value exceeds size
                (
                    0b111,
                    0b100,
                    match size {
                        OpSize::B => vec![(value & 0xFF) as u16],
                        OpSize::W => vec![*value as u16],
                        OpSize::L => vec![(value >> 16) as u16, *value as u16],
                        _ => unreachable!(),
                    },
                )
            }

            AddressingMode::BranchDisplacement(op_size, value) => {
                let size = match op_size {
                    OpSize::B => 0,
                    OpSize::W => 1,
                    _ => todo!(),
                };

                (0, size, vec![value.resolve_value(labels, defines)? as u16])
            }

            AddressingMode::CCR => (CCR_MASK >> 3, 0, vec![]),
            AddressingMode::SR => (SR_MASK >> 3, 0, vec![]),
            AddressingMode::USP => (USP_MASK >> 3, 0, vec![]),
            AddressingMode::RegisterList(mask) => (MOVEM_MASK >> 3, 0, vec![*mask]),
            AddressingMode::DataQuick(imm) => (0, 0, vec![*imm as u16]),
            AddressingMode::Empty => (0b111, 0b111, vec![]),
        };

        Ok(((ea.0 << 3) | ea.1 as u16, ea.2))
    }
}

pub enum AddressingList {
    All,
    Alterable,
    DataAlterable,
    MemoryAlterable,
    Control,
    DataAddressing,
    DataAddressing2,
    Register,
    AddressRegister,
    DataRegister,
    AddressPostincrement,
    AddressPredecrement,
    AddressDisplacement,
    Immediate,

    DataRegisterDataQuick,
    DataQuick,
    Displacement,

    CCR,
    SR,
    USP,

    RegisterList,
    MovemSrc,
    MovemDst,
}

impl AddressingList {
    pub fn check_mode(&self, mode: &AddressingMode) -> bool {
        let list_mask = match self {
            Self::All                   => 0b0_000_000_111111111111,
            Self::Alterable             => 0b0_000_000_011111111111,
            Self::DataAlterable         => 0b0_000_000_011001111101,
            Self::MemoryAlterable       => 0b0_000_000_011001111100,
            Self::Control               => 0b0_000_000_011111100100,
            Self::DataAddressing        => 0b0_000_000_111111111101,
            Self::DataAddressing2       => 0b0_000_000_011111111101,
            Self::Register              => 0b0_000_000_000000000011,
            Self::AddressRegister       => 0b0_000_000_000000000010,
            Self::DataRegister          => 0b0_000_000_000000000001,
            Self::AddressPostincrement  => 0b0_000_000_000000001000,
            Self::AddressPredecrement   => 0b0_000_000_000000010000,
            Self::AddressDisplacement   => 0b0_000_000_000000100000,
            Self::Immediate             => 0b0_000_000_100000000000,

            Self::DataRegisterDataQuick => 0b0_000_100_000000000001,
            Self::DataQuick             => 0b0_000_100_000000000000,
            Self::Displacement          => 0b0_000_001_000000000000,

            Self::CCR                   => 0b0_001_000_000000000000,
            Self::SR                    => 0b0_010_000_000000000000,
            Self::USP                   => 0b0_100_000_000000000000,

            Self::RegisterList          => 0b0_000_010_000000000000,
            Self::MovemSrc              => 0b0_000_000_011001101100,
            Self::MovemDst              => 0b0_000_000_011001110100,
        };

        mode.mask_bit() & list_mask != 0
    }
}

pub fn determine_addressing_mode(token: &str, opcode: &OpType, size: OpSize, last_label: &str) -> Result<AddressingMode, Log> {
    Ok(if token.starts_with("#") {
        //todo: fine to simply pass opsize as imm size?
        match parse_n(&token[1..]) {
            Ok(val) => match opcode {
                OpType::MoveQ | OpType::Rotation(_, _) | OpType::AddSubQ(_) | OpType::Trap => {
                    AddressingMode::DataQuick(val as u8)
                }
                _ => AddressingMode::Immediate(size, val),
            },

            Err(e) => return Err(e),
        }
    } else if token.to_uppercase().starts_with("D") {
        if let Ok(Some(mask)) = movem(token) {
            return Ok(AddressingMode::RegisterList(mask));
        }

        AddressingMode::DataRegister(parse_reg(&token[1..])?)
    } else if token.to_uppercase().starts_with("A") {
        if let Ok(Some(mask)) = movem(token) {
            return Ok(AddressingMode::RegisterList(mask));
        }

        AddressingMode::AddressRegister(parse_reg(&token[1..])?)
    } else if token.starts_with("-(") {
        AddressingMode::AddressPredecrement(parse_reg(&token[3..token.len() - 1])?)
    } else if token.ends_with("+") {
        AddressingMode::AddressPostincrement(parse_reg(&token[2..token.len() - 2])?)
    } else if token.starts_with("(") {
        if &token[1..=1].to_uppercase() == "A" {
            AddressingMode::Address(parse_reg(&token[2..token.len() - 1])?)
        } else {
            let commas: Vec<_> = token.match_indices(",").collect();

            if commas.len() > 0 {
                if token[commas[0].0..token.len()]
                    .to_uppercase()
                    .contains("PC")
                {
                    match commas.len() {
                        1 => {
                            let disp = match parse_n(&token[1..commas[0].0].trim()) {
                                Ok(val) => val as i16,
                                Err(e) => return Err(e),
                            };

                            AddressingMode::PCDisplacement(disp)
                        }

                        2 => {
                            let disp = match parse_n(&token[1..commas[0].0].trim()) {
                                Ok(val) => val as i8,
                                Err(e) => return Err(e),
                            };

                            let third = token[commas[1].0 + 1..token.len() - 1].trim();

                            let reg_type = if third.starts_with('D') {
                                false
                            } else if third.starts_with('A') {
                                true
                            } else {
                                todo!("error")
                            };

                            let reg_size = if third.to_lowercase().ends_with(".w") {
                                false
                            } else if third.to_lowercase().ends_with(".l") {
                                true
                            } else {
                                todo!("error")
                            };

                            AddressingMode::PCIndex(
                                disp,
                                parse_reg(&third[1..=1])?,
                                reg_type,
                                reg_size,
                            )
                        }

                        _ => todo!("error"),
                    }
                } else {
                    match commas.len() {
                        1 => {
                            let disp = match parse_n(&token[1..commas[0].0].trim()) {
                                Ok(val) => val as i16,
                                Err(e) => return Err(e),
                            };

                            let second = token[commas[0].0 + 1..token.len() - 1].trim();
                            let reg = parse_reg(&second[1..])?;
                            AddressingMode::AddressDisplacement(disp, reg)
                        }

                        2 => {
                            //AddressIndex
                            let disp = match parse_n(&token[1..commas[0].0].trim()) {
                                Ok(val) => val as i8,
                                Err(e) => return Err(e),
                            };

                            let second = token[commas[0].0 + 1..commas[1].0].trim();
                            if second.starts_with('A') == false {
                                todo!("error")
                            }

                            let third = token[commas[1].0 + 1..token.len() - 1].trim();

                            let reg_type = if third.starts_with('D') {
                                false
                            } else if third.starts_with('A') {
                                true
                            } else {
                                todo!("error")
                            };

                            let reg_size = if third.to_lowercase().ends_with(".w") {
                                false
                            } else if third.to_lowercase().ends_with(".l") {
                                true
                            } else {
                                return Err(Log::IndexRegisterInvalidSize);
                            };

                            AddressingMode::AddressIndex(
                                disp,
                                parse_reg(&second[1..=1])?,
                                parse_reg(&third[1..=1])?,
                                reg_type,
                                reg_size,
                            )
                        }

                        _ => todo!("error"),
                    }
                }
            } else {
                todo!("error");
            }
        }
    } else if token.to_uppercase() == "CCR" {
        AddressingMode::CCR
    } else if token.to_uppercase() == "SR" {
        AddressingMode::SR
    } else {
        if token.to_lowercase().ends_with(".w") {
            //explicit word
            match parse_n(&token[..token.len() - 2]) {
                Ok(val) => AddressingMode::AbsoluteShort(val as u16),
                Err(e) => return Err(e),
            }
        } else if token.to_lowercase().ends_with(".l") {
            //explicit long
            match parse_n(&token[..token.len() - 2]) {
                Ok(val) => AddressingMode::AbsoluteLong(Value::Number(val)),
                Err(e) => return Err(e),
            }
        } else {
            let value = match parse_n(token) {
                Ok(number) => Value::Number(number),

                Err(_) => {
                    if token.starts_with('!') {
                        Value::Define(token[1..].to_string())
                    } else if token.starts_with('.') {
                        let mut sub_label = last_label.to_string();
                        sub_label.push_str(token);
                        Value::Label(sub_label)
                    } else {
                        Value::Label(token.to_string())
                    }
                }
            };

            match opcode {
                OpType::Branch(_) | OpType::Dbcc(_) => {
                    AddressingMode::BranchDisplacement(size, value)
                }

                _ => {
                    AddressingMode::AbsoluteLong(value) //todo: pick short/long based on value
                }
            }
        }
    })
}

fn movem(token: &str) -> Result<Option<u16>, Log> {
    if token.len() > 2 {
        let mut mask = 0;
        let v: Vec<&str> = token.split('/').collect();

        for &thing in v.iter() {
            if thing.contains('-') {
                let range = {
                    let (a, b) = thing.split_once('-').unwrap();

                    let base = if a.starts_with('D') && b.starts_with('D') {
                        0
                    } else if a.starts_with('A') && b.starts_with('A') {
                        8
                    } else {
                        todo!()
                    };

                    let (x, y) = (parse_reg(&a[1..])? + base, parse_reg(&b[1..])? + base);

                    match x <= y {
                        true  => x..=y,
                        false => y..=x,
                    }
                };

                for x in range {
                    mask |= 1 << x;
                }
            } else {
                let base = if thing.starts_with("D") {
                    0
                } else if thing.starts_with("A") {
                    8
                } else {
                    todo!()
                };

                mask |= 1 << (parse_reg(&thing[1..])? + base);
            }
        }

        Ok(Some(mask))
    } else {
        Ok(None)
    }
}

fn parse_reg(token: &str) -> Result<u8, Log> {
    match token.parse::<u32>() {
        Ok(reg) => {
            if reg > 7 {
                todo!("register out of range");
            }

            Ok(reg as u8)
        }

        Err(_) => Err(Log::InvalidRegister),
    }
}
