use clap::Parser;
use mips_assembler::{assemble, assemble_to_u8, BResult, Endian};

#[derive(Debug, Parser)]
#[clap(name = "mimi", version = "v1.0.0", about = "Minimum mips assembler")]

struct Args {
    /// Store data as big-endian [default: false]
    #[arg(short, long = "big-endian", default_value_t = false)]
    be: bool,

    /// Read assembly from <Input File>.
    #[arg(value_name = "Input File")]
    input: String,

    /// Place the output into <Output File>.
    #[arg(value_name = "Output File", short, long, default_value = "output")]
    output: String,

    /// If it is valid, print the result in text format to standard output.
    #[arg(short = 's', long = "string", default_value_t = false)]
    string: bool,
}

fn main() -> BResult<()> {
    let args = Args::parse();

    let endian = if args.be { Endian::Big } else { Endian::Little };
    if args.string {
        let code = assemble_to_u8(endian, &args.input)?;
        for c in code.chunks(4) {
            println!("{:08b}{:08b}{:08b}{:08b}", c[0], c[1], c[2], c[3]);
        }
    }
    assemble(
        if args.be { Endian::Big } else { Endian::Little },
        args.input,
        args.output,
    )?;

    Ok(())
}
