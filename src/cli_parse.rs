use clap::Parser;

#[derive(Parser)]
pub struct Args {
    /// Path to file to assemble
    #[arg(short, default_value = "code.asm")]
    pub in_file: String,

    /// Path to where to create assembled file. If none is specified, the in_file name will be used, adding or replacing an existing file extension with ".bin"
    #[arg(short)]
    pub out_file: Option<String>,

    /// Valid options are "M68000", "M68010", "M68020"
    #[arg(short, default_value = "M68000")]
    pub target_cpu: String,
}
