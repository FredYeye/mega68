#![allow(clippy::unusual_byte_groupings)]

mod addressing;
mod opsize;
mod optype;
mod value;
mod constants;

use crate::{logging::Log, assembler::constants::*};

use addressing::AddressingMode;
use opsize::OpSize;
use optype::OpType;

use std::collections::HashMap;

use self::value::Value;

#[derive(Debug)]
struct TokenizedString {
    opcode: String,
    size: Option<String>,
    operands: [Option<String>; 2],
}

#[derive(Debug)]
struct Decoded { //todo: rename
    op_type: OpType,
    op_size: OpSize,
    operands: [addressing::AddressingMode; 2],
    line: u32,
    location: u32,
}

#[derive(Default)]
enum CpuType {
    #[default] MC68000,
    MC68010,
}

#[derive(Debug)]
enum DataType {
    Data08,
    Data16,
    Data24,
    Data32,
    Data64,
}

impl DataType {
    fn is_data(token: &str) -> Option<Self> {
        match token {
            "d08" => Some(Self::Data08),
            "d16" => Some(Self::Data16),
            "d24" => Some(Self::Data24),
            "d32" => Some(Self::Data32),
            "d64" => Some(Self::Data64),
            _ => None,
        }
    }
}

#[derive(Default)]
pub struct Assembler {
    tokens: Vec<Decoded>,
    assembled: Vec<u16>,
    location: u32,
    line: u32,
    labels: HashMap<String, u32>,
    last_label: String,
    defines: HashMap<String, u64>,
    cpu_type: CpuType,
}

impl Assembler {
    pub fn run(&mut self, text: &str) -> Result<&Vec<u16>, (Log, u32)> {
        if let Err(e) = self.tokenize_string(text) {
            return Err((e, self.line));
        }

        for token in &self.tokens {
            // println!("{:?}", token);

            match self.assemble(token) {
                Ok(o) => self.assembled.extend(o),
                Err(e) => return Err((e, token.line)),
            }
        }

        Ok(&self.assembled)
    }

    fn tokenize_string(&mut self, text: &str) -> Result<(), Log> {
        for lines in text.lines() {
            self.line += 1;

            let trimmed_str = lines.split(';').next().unwrap().trim();
            let separated_op: Vec<&str> = trimmed_str.splitn(2, ' ').collect();

            if separated_op[0].is_empty() {
                continue;
            }

            if let Some(data_type) = DataType::is_data(separated_op[0]) {
                self.data_define(separated_op[1], data_type)?;
                continue;
            }

            if let Some(label) = separated_op[0].strip_suffix(':') {
                self.label_define(label)?;
                continue;
            }

            if let Some(define_name) = separated_op[0].strip_prefix('!') {
                let define_val = separated_op[1].trim_start();

                if let Some(define_val) = define_val.strip_prefix('=') {
                    let val = parse_n(define_val.trim_start())?;
                    self.defines.insert(define_name.to_string(), val);
                }

                continue;
            }

            let (opcode, size) = if let Some((op, suffix)) = separated_op[0].split_once('.') {
                (op.to_string(), Some(suffix.to_string()))
            } else {
                (separated_op[0].to_string(), None)
            };

            let mut operands = [None, None];

            if separated_op.len() > 1 {
                let commas: Vec<_> = separated_op[1].match_indices(',').collect();

                if commas.is_empty() { //one operand
                    operands[0] = Some(separated_op[1].trim_start().to_string());
                } else { //one or two operands
                    let parentheses = (separated_op[1].find('('), separated_op[1].find(')'));

                    match parentheses {
                        (None, None) => { //separator comma
                            let split_comma: Vec<&str> = separated_op[1].split(',').collect();
                            operands[0] = Some(split_comma[0].trim().to_string());
                            operands[1] = Some(split_comma[1].trim_start().to_string());
                        }

                        (Some(paren0), Some(paren1)) => {
                            for x in 0..commas.len() {
                                let parentheses_pos = (
                                    paren0 < commas[x].0,
                                    paren1 < commas[x].0,
                                );

                                match parentheses_pos {
                                    (true, true) | (false, false) => { // (), | ,() separator comma
                                        let split_comma = separated_op[1].split_at(commas[x].0);
                                        operands[0] = Some(split_comma.0.trim().to_string());
                                        operands[1] = Some(split_comma.1[1..].trim_start().to_string());
                                        break;
                                    }

                                    (true, false) => { // (,) inside parentheses, go to next comma
                                        if x == commas.len() - 1 { //last comma inside parentheses, one operand
                                            operands[0] = Some(separated_op[1].trim_start().to_string());
                                        }
                                    }

                                    (false, true) => todo!("mismatched parentheses"), // ),( error
                                }
                            }
                        }

                        (Some(_), None) | (None, Some(_)) => todo!("mismatched parentheses"),
                    }
                }
            }

            let string_token = TokenizedString {
                opcode,
                size,
                operands,
            };

            match self.string_token_to_token(&string_token, self.line, self.location) {
                Ok(token) => {
                    self.location += match &token.op_type {
                        // OpType::Data(data) => data.len() as u32 * 2, //unused!
                        _ => 2 + AddressingMode::ea_size(&token.operands) as u32,
                    };

                    self.tokens.push(token)
                }

                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    fn string_token_to_token(&mut self, tokens: &TokenizedString, line: u32, location: u32) -> Result<Decoded, Log> {
        let opcode = OpType::parse_op(&tokens.opcode)?;

        let size = if let Some(size_suffix) = &tokens.size {
            match size_suffix as &str {
                "b" => OpSize::B,
                "w" => OpSize::W,
                "l" => OpSize::L,
                _ => return Err(Log::InvalidSuffix),
            }
        } else {
            OpSize::Unsized
        };

        let mut modes = [AddressingMode::Empty, AddressingMode::Empty];

        for (x, mode) in modes.iter_mut().enumerate() {
            *mode = match &tokens.operands[x] {
                Some(operand) => addressing::determine_addressing_mode(operand, &opcode, size, &self.last_label)?,
                None => AddressingMode::Empty,
            };

            if let AddressingMode::AddressRegister(_) = mode {
                if size == OpSize::B {
                    return Err(Log::AnB);
                }
            }
        }

        Ok(Decoded {
            op_type: opcode,
            op_size: size,
            operands: modes,
            line: line,
            location: location,
        })
    }

    fn label_define(&mut self, label: &str) -> Result<(), Log> {
        if label.starts_with('.') { // sub label
            let sub_label = format!("{}{}", self.last_label, label);
            
            if !self.labels.contains_key(&sub_label) {
                self.labels.insert(sub_label, self.location);
            } else {
                return Err(Log::LabelRedefinition);
            }
        } else {
            if !self.labels.contains_key(label) {
                self.last_label = label.to_string();
                self.labels.insert(label.to_string(), self.location);
            } else {
                return Err(Log::LabelRedefinition);
            }
        }

        Ok(())
    }

    fn data_define(&mut self, list: &str, size: DataType) -> Result<(), Log> {
        let mut vec = Vec::new();

        for data in list.split(',').map(|x| x.trim()) {
            vec.push(Value::new(data, &self.last_label));
        }

        let mut len = vec.len() * match size {
            DataType::Data08 => 1,
            DataType::Data16 => 2,
            DataType::Data24 => 3,
            DataType::Data32 => 4,
            DataType::Data64 => 8,
        };

        if len & 1 != 0 {
            // probably print a warning here
            len += 1;
        }

        self.tokens.push(Decoded {
            op_type: OpType::Data(size, vec),
            op_size: OpSize::Unsized,
            operands: [AddressingMode::Empty, AddressingMode::Empty],
            line: self.line,
            location: self.location,
        });

        self.location += len as u32;

        Ok(())
    }

    fn assemble(&self, op: &Decoded) -> Result<Vec<u16>, Log> {
        use OpSize::*;
        use OpType::*;

        op.op_type.valid_size(op.op_size)?;

        op.op_type.is_valid_modes(&op.operands)?;

        let (ea_a1, ea_a2) = op.operands[0].effective_addressing(&self.labels, &self.defines, op.location)?;
        let (ea_b1, ea_b2) = op.operands[1].effective_addressing(&self.labels, &self.defines, op.location)?;

        Ok(match &op.op_type {
            Branch(_) => {
                match op.op_size {
                    B => vec![op.op_type.format() | (ea_a2[0] & 0xFF)],
                    W => vec![op.op_type.format(), ea_a2[0]],
                    _ => unreachable!(),
                }
            }

            NoOperands(_) => vec![op.op_type.format()],

            Immediates(_) => {
                let ea = match ea_b1 {
                    CCR_MASK => {
                        if op.op_size == B {
                            IMMEDIATE_MASK
                        } else {
                            todo!("error")
                        }
                    }

                    SR_MASK => {
                        if op.op_size == W {
                            IMMEDIATE_MASK
                        } else {
                            todo!("error")
                        }
                    }

                    _ => ea_b1,
                };

                let mut format = vec![op.op_type.format() | op.op_size.size1() | ea];
                format.extend(ea_a2);
                format.extend(ea_b2);
                format
            }

            AddSub(_) | OrAnd(_) => {
                let (reg, dir, ea) = match ea_b1 & MODE_MASK == 0b000_000 {
                    true => (ea_b1 & 0b111, 0, (ea_a1, ea_a2)),
                    false => (ea_a1 & 0b111, 1 << 8, (ea_b1, ea_b2)),
                };

                let mut format = vec![op.op_type.format() | (reg << 9) | dir | op.op_size.size1() | ea.0];
                format.extend(ea.1);
                format
            }

            Eor => {
                let reg = ea_a1 & 0b111;
                let mut format = vec![op.op_type.format() | (reg << 9) | op.op_size.size1() | ea_b1];
                format.extend(ea_b2);
                format
            }

            AddSubA(_) => {
                let reg = (ea_b1 & 0b111) << 9;

                let opmode = match op.op_size {
                    OpSize::W => 0b011 << 6,
                    OpSize::L => 0b111 << 6,
                    _ => unreachable!(),
                };

                let mut format = vec![op.op_type.format() | reg | opmode | ea_a1];
                format.extend(ea_a2);
                format
            }

            AddSubX(_) | Bcd(_) => {
                let rm = match ea_a1 & MODE_MASK == 0b000_000 {
                    true => 0,
                    false => 1 << 3,
                };

                let rx = (ea_a1 & 0b111) << 9;
                let ry = ea_b1 & 0b111;

                vec![op.op_type.format() | rx | (1 << 8) | rm | ry]
            }

            AddSubQ(_) => {
                let imm = ea_a2[0];

                if !(1..=8).contains(&imm) {
                    todo!("addq/subq immediate out of range")
                }

                if ea_b1 & MODE_MASK == ADDRESS_REGISTER_MASK && op.op_size == OpSize::W {
                    println!("info: addq.w/subq.w will operate on the entire address register");
                }

                let mut format = vec![op.op_type.format() | ((imm & 0b111) << 9) | op.op_size.size1() | ea_b1];
                format.extend(ea_b2);
                format
            }

            Jump(_) | Tas | Pea | Scc(_) | Nbcd => {
                let mut format = vec![op.op_type.format() | ea_a1];
                format.extend(ea_a2);
                format
            }

            Move => {
                match ea_a1 {
                    SR_MASK => {
                        if op.op_size == W {
                            let mut format = vec![(0b0100_0000_11 << 6) | ea_b1];
                            format.extend(ea_b2);
                            format
                        } else {
                            todo!("error")
                        }
                    }

                    USP_MASK => match op.op_size == L {
                        true  => vec![(0b0100_11100110) | ea_b1],
                        false => todo!("error"),
                    }

                    _ => {
                        match ea_b1 {
                            CCR_MASK => {
                                if op.op_size == B {
                                    let mut format = vec![(0b0100_0100_11 << 6) | ea_a1];
                                    format.extend(ea_a2);
                                    format
                                } else {
                                    todo!("error")
                                }
                            }

                            SR_MASK => {
                                if op.op_size == W {
                                    let mut format = vec![(0b0100_0110_11 << 6) | ea_a1];
                                    format.extend(ea_a2);
                                    format
                                } else {
                                    todo!("error")
                                }
                            }

                            USP_MASK => {
                                if op.op_size == L {
                                    let reg = ea_a1 & 0b111;
                                    vec![(0b0100_11100110) | reg]
                                } else {
                                    todo!("error")
                                }
                            }

                            _ => { //normal move
                                let ea_dst_reorder = ((ea_b1 & 0b000_111) << 9) | ((ea_b1 & 0b111_000) << 3);
                                let mut format = vec![op.op_size.size_move() | ea_dst_reorder | ea_a1];
                                format.extend(ea_a2);
                                format.extend(ea_b2);
                                format
                            }
                        }
                    }
                }
            }

            MoveA => {
                let reg = (ea_b1 & 0b111) << 9;
                let mut format = vec![op.op_size.size_move() | reg | op.op_type.format() | ea_a1];
                format.extend(ea_a2);
                format
            }

            BitManip(_) => {
                let op1 = match ea_a1 & MODE_MASK == 0b000_000 {
                    true => (((ea_a1 & 0b111) << 9) | (1 << 8), vec![]),
                    false => {
                        let asd = match ea_b1 & MODE_MASK == DATA_REGISTER_MASK {
                            true  => 1,
                            false => 0,
                        };

                        ((1 << 11), vec![(ea_a2[asd] & 0xFF) as u16])
                    }
                };

                if op.op_size == B && ea_b1 & MODE_MASK == 0b000_000 {
                    todo!("error")
                } else if op.op_size == L && ea_b1 & MODE_MASK != 0b000_000 {
                    todo!("error")
                }

                let mut format = vec![op.op_type.format() | op1.0 | ea_b1];
                format.extend(op1.1);
                format.extend(ea_b2);
                format
            }

            Misc1(_) | Tst => {
                match (&op.op_type, &self.cpu_type) {
                    (Tst, CpuType::MC68000 | CpuType::MC68010) => {
                        if ea_a1 & MODE_MASK == ADDRESS_REGISTER_MASK || ea_a1 == IMMEDIATE_MASK || ea_a1 == 0b111_010 || ea_a1 == 0b111_011 {
                            return Err(Log::CpuTypeModeNotValid);
                        }
                    }

                    _ => (),
                }

                let mut format = vec![op.op_type.format() | op.op_size.size1() | ea_a1];
                format.extend(ea_a2);
                format
            }

            MoveQ => {
                let data = ea_a2[0] & 0b111;
                let reg = (ea_b1 & 0b111) << 9;
                vec![op.op_type.format() | reg | data]
            }

            Rotation(rot_type, dir) => {
                let mode = match ea_b1 & MODE_MASK == 0b000_000 {
                    true => {
                        let (count_reg, ir) = match ea_a1 & MODE_MASK == 0b000_000 {
                            true => (ea_a1 & 0b111, true),
                            false => (ea_a2[0] & 0b111, false),
                        };

                        let mut bits = (ea_b1 & 0b111) | ((*rot_type as u16) << 3);
                        bits |= op.op_size.size1();
                        bits |= (count_reg as u16) << 9;
                        bits |= (ir as u16) << 5;
                        (bits, vec![])
                    }

                    false => {
                        let mut bits = ea_a1;
                        bits |= 0b11 << 6;
                        bits |= (*rot_type as u16) << 9;

                        (bits, ea_a2)
                    }
                };

                let mut format = vec![op.op_type.format() | ((*dir as u16) << 8) | mode.0];
                format.extend(mode.1);
                format
            }

            Lea | Chk | MulDiv(_) => {
                let reg = (ea_b1 & 0b111) << 9;
                let mut format = vec![op.op_type.format() | reg | ea_a1];
                format.extend(ea_a2);
                format
            }

            Exg => {
                let (mode, ry) = match ea_b1 & MODE_MASK == 0b000_000 {
                    true => (0b01000 << 3, (ea_b1 & 0b111)),

                    false => {
                        let op_mode = match ea_a1 & MODE_MASK == 0b000_000 {
                            true => 0b10001 << 3,
                            false => 0b01001 << 3,
                        };

                        (op_mode, ea_a1 & 0b111)
                    }
                };

                let rx = (ea_a1 & 0b111) << 9;

                vec![op.op_type.format() | rx | mode | ry]
            }

            Ext => {
                let reg = ea_a1 & 0b111;
                let size = op.op_size.size2();
                vec![op.op_type.format() | size | reg]
            }

            Swap | Unlk => {
                let reg = ea_a1 & 0b111;
                vec![op.op_type.format() | reg]
            }

            Link => {
                let reg = ea_a1 & 0b111;
                let mut format = vec![op.op_type.format() | reg];
                format.extend(ea_b2);
                format
            }

            Trap => {
                let vector = ea_a2[0] & 0b1111;
                vec![op.op_type.format() | vector]
            }

            Stop => {
                let mut format = vec![op.op_type.format()];
                format.extend(ea_a2);
                format
            }

            Cmp => {
                let reg = (ea_b1 & 0b111) << 9;
                let mut format = vec![op.op_type.format() | reg | op.op_size.size1() | ea_a1];
                format.extend(ea_a2);
                format
            }

            Cmpa => {
                let reg = (ea_b1 & 0b111) << 9;
                let mut format = vec![op.op_type.format() | reg | op.op_size.size3() | ea_a1];
                format.extend(ea_a2);
                format
            }

            Cmpm => {
                let ay = ea_a1 & 0b111;
                let ax = (ea_b1 & 0b111) << 9;
                vec![op.op_type.format() | ax | op.op_size.size1() | ay]
            }

            Dbcc(_) => {
                let disp = (ea_b2[0] as i16 - (op.location as i16 + 2)) as u16;
                let reg = ea_a1 & 0b111;
                vec![op.op_type.format() | reg, disp]
            }

            Movep => {
                let (reg, dir, ea) = match ea_a1 & MODE_MASK == 0b000_000 {
                    true => ((ea_a1 & 0b111) << 9, 1 << 7, (ea_b1, ea_b2)),
                    false => ((ea_b1 & 0b111) << 9, 0, (ea_a1, ea_a2)),
                };

                let mut format =
                    vec![op.op_type.format() | reg | dir | op.op_size.size2() | (ea.0 & 0b111)];
                format.extend(ea.1);
                format
            }

            Movem => {
                let (dr, mask, ea) = if ea_a1 == MOVEM_MASK {
                    let mask = match ea_b1 & MODE_MASK == 0b100_000 {
                        true  => ea_a2[0].reverse_bits(),
                        false => ea_a2[0],
                    };

                    (0 << 10, mask, (ea_b1, ea_b2))
                } else {
                    (1 << 10, ea_b2[0], (ea_a1, ea_a2))
                };

                let mut format = vec![op.op_type.format() | dr | op.op_size.size2() | ea.0];
                format.extend(vec![mask]);
                format.extend(ea.1);
                format
            }

            Data(size, values) => {
                let mut vec = Vec::new();

                for value in values {
                    let number = value.resolve_value(&self.labels, &self.defines)?.to_be_bytes();

                    let range = match size {
                        DataType::Data08 => 7..=7,
                        DataType::Data16 => 6..=7,
                        DataType::Data24 => 5..=7,
                        DataType::Data32 => 4..=7,
                        DataType::Data64 => 0..=7,
                    };

                    vec.extend_from_slice(&number[range]);
                }

                if vec.len() & 1 != 0 {
                    vec.push(0);
                }

                let vec2: Vec<u16> = vec
                    .chunks_exact(2)
                    .into_iter()
                    .map(|x| u16::from_be_bytes([x[0], x[1]]))
                    .collect();

                vec2
            }
        })
    }
}

fn parse_n(token: &str) -> Result<u64, Log> {
    let (radix, offset_begin) = if token.len() > 2 {
        match &token[0..2] {
            "0x" => (16, 2),
            "0b" => (2, 2),
            _ => (10, 0),
        }
    } else {
        (10, 0)
    };

    match i64::from_str_radix(&token[offset_begin..token.len()], radix) {
        Ok(val) => Ok(val as u64),

        Err(_) => match u64::from_str_radix(&token[offset_begin..token.len()], radix) {
            Ok(val) => Ok(val),
            Err(_) => Err(Log::InvalidNumber),
        }
    }
}
