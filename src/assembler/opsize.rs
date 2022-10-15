#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpSize {
    B, W, L,
    BW, BL, WL,
    BWL,

    Unsized,
    WU,
    LU,
}

impl OpSize {
    pub fn mask(&self) -> u8 {
        match self {
            OpSize::B   => 0b001,
            OpSize::W   => 0b010,
            OpSize::L   => 0b100,

            OpSize::BW  => 0b011,
            OpSize::BL  => 0b101,
            OpSize::WL  => 0b110,

            OpSize::BWL => 0b111,

            OpSize::Unsized => 0b1000,
            OpSize::WU => 0b1010,
            OpSize::LU => 0b1100,
        }
    }

    pub fn size1(&self) -> u16 {
        match self {
            OpSize::B => 0b00 << 6,
            OpSize::W => 0b01 << 6,
            OpSize::L => 0b10 << 6,
            _ => unreachable!(),
        }
    }

    pub fn size2(&self) -> u16 {
        match self {
            OpSize::W => 0 << 6,
            OpSize::L => 1 << 6,
            _ => unreachable!(),
        }
    }

    pub fn size3(&self) -> u16 {
        match self {
            OpSize::W => 0 << 8,
            OpSize::L => 1 << 8,
            _ => unreachable!(),
        }
    }

    pub fn size_move(&self) -> u16 {
        match self {
            OpSize::B => 0b01 << 12,
            OpSize::W => 0b11 << 12,
            OpSize::L => 0b10 << 12,
            _ => unreachable!(),
        }
    }
}
