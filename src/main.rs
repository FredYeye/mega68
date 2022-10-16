mod assembler;

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
}

impl Log {
    fn print(&self) -> &str {
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
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let file = if args.len() > 1 {
        &args[1]
    } else {
        "code.asm"
    };

    let mut asm = assembler::Assembler::default();

    let text = std::fs::read_to_string(file).expect("couldn't read file");

    match asm.run(&text) {
        Ok(assembled) => {
            println!("{:04X?}", assembled);

            let mut u8_vec = Vec::new();

            for elem in assembled {
                u8_vec.extend(elem.to_be_bytes());
            }

            std::fs::write("assembled.bin", u8_vec).expect("unable to write file");
        }

        Err((err_type, l)) => println!("Line {}: {}", l, err_type.print()),
    }
}
