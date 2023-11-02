use assembler::CpuType;

mod assembler;
mod logging;
mod tests;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    //todo: update command line options handling
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

    let target_cpu = if args.len() > 3 {
        CpuType::MC68010
    } else {
        CpuType::MC68000
    };

    let mut asm = assembler::Assembler::default();
    asm.cpu_type = target_cpu;

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
