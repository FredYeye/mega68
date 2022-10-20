mod assembler;
mod logging;

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
