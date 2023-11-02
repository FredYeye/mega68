// struct Logging {
//     warn: Vec<(u32, LogWarn)>,
// }

#[derive(Debug, PartialEq)]
pub enum Log {
    InvalidOp,
    InvalidSuffix,
    AnB,
    UnsupportedSuffix,
    IndexRegisterInvalidSize,
    InvalidNumber,
    NoLabel,
    NoDefine,
    InvalidRegister,
    InvalidAddressingMode,
    TooManyOperands,
    LabelRedefinition,
    CpuTypeModeNotValid,
    SizeOperandMismatch,
    UnsupportedInstruction,
}

impl Log {
    pub fn print(&self) -> &str {
        match self {
            Self::InvalidOp => "Invalid opcode",
            Self::InvalidSuffix => "Invalid size suffix",
            Self::AnB => "Byte operations on address registers are invalid",
            Self::UnsupportedSuffix => "This opcode does not support this size",
            Self::IndexRegisterInvalidSize => "Index register size is either invalid or missing",
            Self::InvalidNumber => "Failed to parse number",
            Self::NoLabel => "Label doesn't exist",
            Self::NoDefine => "Define doesn't exist",
            Self::InvalidRegister => "Invalid register specified",
            Self::InvalidAddressingMode => "Invalid addressing mode",
            Self::TooManyOperands => "Expected one operand, found two",
            Self::LabelRedefinition => "Label redefinition",
            Self::CpuTypeModeNotValid => "This addressing mode is not valid for this CPU type",
            Self::SizeOperandMismatch => "invalid size / operand combination",
            Self::UnsupportedInstruction => "Target CPU does not support this instruction",
        }
    }
}
