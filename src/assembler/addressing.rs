#![allow(clippy::upper_case_acronyms)]

use crate::{logging::Log};

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
    Immediate(OpSize, Value),

    BranchDisplacement(OpSize, Value),
    RegisterList(u16),
    DataQuick(Value),

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
                Self::AddressDisplacement(_, _) | Self::AddressIndex(_, _, _, _, _) |
                Self::PCDisplacement(_)         | Self::PCIndex(_, _, _, _) |
                Self::AbsoluteShort(_)          | Self::RegisterList(_) => 2,

                Self::AbsoluteLong(_) => 4,

                Self::Immediate(size, _) => match size {
                    OpSize::L => 4,
                    _ => 2,
                },

                Self::BranchDisplacement(OpSize::W, _) => 2,

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
            Self::DataRegister(reg) => (0b000, *reg, vec![]),
            Self::AddressRegister(reg) => (0b001, *reg, vec![]),

            Self::Address(reg) => (0b010, *reg, vec![]),
            Self::AddressPostincrement(reg) => (0b011, *reg, vec![]),
            Self::AddressPredecrement(reg) => (0b100, *reg, vec![]),
            Self::AddressDisplacement(disp, reg) => (0b101, *reg, vec![*disp as u16]),

            Self::AddressIndex(disp, reg_a, reg, reg_type, reg_size) => (
                0b110,
                *reg_a,
                vec![((*reg_type as u16) << 15) | ((*reg as u16) << 12) | ((*reg_size as u16) << 11) | *disp as u16],
            ),

            Self::PCDisplacement(disp) => (0b111, 0b010, vec![*disp as u16]),
            Self::PCIndex(disp, reg, reg_type, reg_size) => (
                0b111,
                0b011,
                vec![((*reg_type as u16) << 15) | ((*reg as u16) << 12) | ((*reg_size as u16) << 11) | *disp as u16],
            ),

            Self::AbsoluteShort(addr) => (0b111, 0b000, vec![*addr]),
            Self::AbsoluteLong(value) => {
                let addr = value.resolve_value(labels, defines)?;
                (0b111, 0b001, vec![(addr >> 16) as u16, addr as u16])
            }

            Self::Immediate(size, value) => {
                //todo: warn if value exceeds size
                let val = value.resolve_value(labels, defines)?;

                (
                    0b111,
                    0b100,
                    match size {
                        OpSize::B => vec![(val & 0xFF) as u16],
                        OpSize::W => vec![val as u16],
                        OpSize::L => vec![(val >> 16) as u16, val as u16],
                        _ => unreachable!(),
                    },
                )
            }

            Self::BranchDisplacement(op_size, value) => {
                let size = match op_size {
                    OpSize::B | OpSize::Unsized => 0, //let unsized through for dbcc
                    OpSize::W => 1,
                    _ => todo!(),
                };

                (0, size, vec![value.resolve_value(labels, defines)? as u16])
            }

            Self::CCR => (CCR_MASK >> 3, 0, vec![]),
            Self::SR => (SR_MASK >> 3, 0, vec![]),
            Self::USP => (USP_MASK >> 3, 0, vec![]),
            Self::RegisterList(mask) => (MOVEM_MASK >> 3, 0, vec![*mask]),
            Self::DataQuick(imm) => (0, 0, vec![(imm.resolve_value(labels, defines)? & 0xFF) as u16]),
            Self::Empty => (0b111, 0b111, vec![]),
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
    pub fn contains(&self, mode: &AddressingMode) -> bool {
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
    use AddressingMode::*;

    if token.len() == 2 {
        match parse_reg(&token[1..=1]) {
            Ok(reg) => {
                if token.to_uppercase().starts_with('D') {
                    return Ok( match opcode {
                        OpType::Movem => RegisterList(1 << reg),
                        _ => DataRegister(reg),
                    })
                } else if token.to_uppercase().starts_with('A') {
                    return Ok( match opcode {
                        OpType::Movem => RegisterList(1 << (reg + 8)),
                        _ => AddressRegister(reg),
                    })
                }
            }

            Err(_) => (),
        }
    }

    if let Some(imm) = token.strip_prefix('#') {
        let val = Value::new(imm, last_label);

        Ok( match opcode {
            OpType::MoveQ | OpType::Rotation(_, _) | OpType::AddSubQ(_) | OpType::Trap => DataQuick(val),
            _ => Immediate(size, val),
        })
    } else if let Some(predec) = token.strip_prefix("-(A") {
        if let Some(predec) = predec.strip_suffix(')') {
            Ok(AddressPredecrement(parse_reg(predec)?))
        } else {
            todo!("error")
        }
    } else if let Some(postinc) = token.strip_suffix(")+") {
        if let Some(postinc) = postinc.strip_prefix("(A") {
            Ok(AddressPostincrement(parse_reg(postinc)?))
        } else {
            todo!("error")
        }
    } else if let Some(paren_token) = token.strip_prefix('(') {
        if let Some(paren_token) = paren_token.strip_suffix(')') {
            let commas: Vec<_> = paren_token.match_indices(',').collect();

            match commas.len() {
                0 => {
                    if paren_token.len() == 2 && paren_token.to_uppercase().starts_with('A') {
                        Ok(Address(parse_reg(&paren_token[1..=1])?))
                    } else {
                        todo!("error")
                    }
                }

                1 => {
                    let disp = match parse_n(paren_token[..commas[0].0].trim()) {
                        Ok(val) => val as i16,
                        Err(e) => return Err(e),
                    };

                    let second = paren_token[commas[0].0 + 1..paren_token.len()].trim();

                    if second == "PC" {
                        Ok(PCDisplacement(disp))
                    } else if second.len() == 2 && second.starts_with('A') {
                        Ok(AddressDisplacement(disp, parse_reg(&second[1..=1])?))
                    } else {
                        todo!("error")
                    }
                }

                2 => {
                    let disp = match parse_n(paren_token[..commas[0].0].trim()) {
                        Ok(val) => val as i8,
                        Err(e) => return Err(e),
                    };

                    let third = paren_token[commas[1].0 + 1..paren_token.len()].trim();

                    let (reg_type, reg_num, reg_size) = if third.len() == 4 {
                        (
                            if third.starts_with('D') {
                                false
                            } else if third.starts_with('A') {
                                true
                            } else {
                                todo!("error")
                            },
    
                            parse_reg(&third[1..=1])?,

                            if third.to_lowercase().ends_with(".w") {
                                false
                            } else if third.to_lowercase().ends_with(".l") {
                                true
                            } else {
                                return Err(Log::IndexRegisterInvalidSize)
                            },
                        )
                    } else {
                        todo!("error")
                    };

                    let second = paren_token[commas[0].0 + 1..commas[1].0].trim();

                    if second == "PC" {
                        Ok(PCIndex(disp, reg_num, reg_type, reg_size))
                    } else if second.len() == 2 && second.starts_with('A') {
                        Ok(AddressIndex(disp, parse_reg(&second[1..=1])?, reg_num, reg_type, reg_size))
                    } else {
                        todo!("error")
                    }
                }

                _ => todo!("error")
            }
        } else {
            todo!("error")
        }
    } else if token.to_uppercase() == "CCR" {
        Ok(CCR)
    } else if token.to_uppercase() == "SR" {
        Ok(SR)
    } else if token.to_uppercase() == "USP" {
        Ok(USP)
    } else if token.to_lowercase().ends_with(".w") {
        match parse_n(&token[..token.len() - 2]) {
            Ok(val) => Ok(AbsoluteShort(val as u16)),
            Err(e) => return Err(e),
        }
    } else if token.to_lowercase().ends_with(".l") {
        match parse_n(&token[..token.len() - 2]) {
            Ok(val) => Ok(AbsoluteLong(Value::Number(val as u32))),
            Err(e) => return Err(e),
        }
    } else {
        match opcode {
            OpType::Movem => match movem(token) {
                Ok(mask) => Ok(RegisterList(mask)),
                Err(e) => return Err(e),
            }

            OpType::Branch(_) | OpType::Dbcc(_) => Ok(BranchDisplacement(size, Value::new(token, last_label))),

            _ => {
                Ok(AbsoluteLong(Value::new(token, last_label))) //todo: pick short/long based on value
            }
        }
    }
}

fn movem(token: &str) -> Result<u16, Log> {
    let mut mask = 0;
    let v: Vec<&str> = token.split('/').collect();

    for &section in v.iter() {
        if section.contains('-') {
            let range = {
                let (a, b) = section.split_once('-').unwrap();

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
            let base = if section.starts_with('D') {
                0
            } else if section.starts_with('A') {
                8
            } else {
                todo!()
            };

            mask |= 1 << (parse_reg(&section[1..])? + base);
        }
    }

    Ok(mask)
}

fn parse_reg(token: &str) -> Result<u8, Log> {
    match token.parse::<u32>() {
        Ok(reg) => match reg < 8 {
            true  => Ok(reg as u8),
            false => Err(Log::InvalidRegister),
        }

        Err(_) => Err(Log::InvalidRegister),
    }
}
