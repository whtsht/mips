use clap::Parser;
use mips_assembler::{assemble, AResult, Endian};

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
}

fn main() -> AResult<()> {
    let args = Args::parse();
    assemble(
        if args.be { Endian::Big } else { Endian::Little },
        args.input,
        args.output,
    )?;

    Ok(())
}
