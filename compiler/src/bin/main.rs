use anyhow::Result;
use clap::Parser;
use mips_compiler::compile_from_path;

#[derive(Debug, Parser)]
struct Args {
    input: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("{}", compile_from_path(args.input)?);

    Ok(())
}
