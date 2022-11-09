mod assembler;
mod logging;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let file_in = if args.len() > 1 {
        &args[1]
    } else {
        "code.asm"
    };

    let file_out = if args.len() > 2 {
        args[2].clone()
    } else {
        let name = match file_in.rsplit_once('.') {
            Some((file_name, _)) => file_name,
            None => file_in,
        };

        format!("{name}.bin")
    };

    let mut asm = assembler::Assembler::default();

    let text = std::fs::read_to_string(file_in).expect("couldn't read file");

    match asm.run(&text) {
        Ok(assembled) => {
            println!("{:04X?}", assembled);

            let mut u8_vec = Vec::new();

            for elem in assembled {
                u8_vec.extend(elem.to_be_bytes());
            }

            std::fs::write(file_out, u8_vec).expect("unable to write file");
        }

        Err((err_type, l)) => println!("Line {}: {}", l, err_type.print()),
    }
}

// ----- tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tester() -> Result<(), (logging::Log, u32)> {
        let mut asm = assembler::Assembler::default();
        assert_eq!(asm.run("nop")?, &vec![0x4E71]);
        Ok(())
    }

    #[test]
    fn tester2() -> Result<(), (logging::Log, u32)> {
        let mut asm = assembler::Assembler::default();
        assert_eq!(asm.run("btst.l #2, D0")?, &vec![0x0800, 0x0002]);
        Ok(())
    }
}
