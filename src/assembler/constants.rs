pub const CCR_MASK:   u16 = 0b0001_000_000;
pub const SR_MASK:    u16 = 0b0010_000_000;
pub const USP_MASK:   u16 = 0b0100_000_000;
pub const MOVEM_MASK: u16 = 0b1000_000_000;

pub const MODE_MASK: u16 = 0b111_000;
pub const DATA_REGISTER_MASK: u16 = 0b000_000;
pub const ADDRESS_REGISTER_MASK: u16 = 0b001_000;
pub const IMMEDIATE_MASK: u16 = 0b111_100;
