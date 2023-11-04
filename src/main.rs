use clap::Parser;

use assembler::CpuType;

mod assembler;
mod logging;
mod tests;

#[derive(Parser)]
struct Args {
    /// Path to file to assemble
    #[arg(short, default_value = "code.asm")]
    in_file: String,

    /// Path to where to create assembled file. If none is specified, the in_file name will be used, adding or replacing an existing file extension with ".bin"
    #[arg(short)]
    out_file: Option<String>,

    /// Valid options are "M68000", "M68010"
    #[arg(short, default_value = "M68000")]
    target_cpu: String,
}

fn main() {
    let args = Args::parse();

    let out_file = match args.out_file {
        Some(s) => s,

        None => {
            let name = match args.in_file.rsplit_once('.') {
                Some((file_name, _)) => file_name,
                None => &args.in_file,
            };
    
            format!("{name}.bin")
        }
    };

    let target_cpu = match args.target_cpu.as_str() {
        "M68000" => CpuType::MC68000,
        "M68010" => CpuType::MC68010,

        _ => {
            println!("Invalid cpu type specified");
            return;
        }
    };

    let mut asm = assembler::Assembler::default();
    asm.cpu_type = target_cpu;

    let text = std::fs::read_to_string(args.in_file).expect("couldn't read file");

    match asm.run(&text) {
        Ok(assembled) => {
            println!("{:04X?}", assembled);

            let mut u8_vec = Vec::new();

            for elem in assembled {
                u8_vec.extend(elem.to_be_bytes());
            }

            std::fs::write(out_file, u8_vec).expect("unable to write file");
        }

        Err((err_type, l)) => println!("Line {}: {}", l, err_type.print()),
    }
}
