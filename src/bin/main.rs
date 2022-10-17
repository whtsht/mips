use clap::Parser;
use mips_assembler::assemble_to_u8;
use mips_emulator::Emulator;

#[derive(Debug, Parser)]
#[clap(name = "mips", version = "v1.0.0", about = "Minimum mips emulator")]
struct Args {
    /// Read assembly from <Input File>
    #[arg(value_name = "Input File")]
    input: String,
}

fn main() {
    let args = Args::parse();
    let bin = assemble_to_u8(mips_assembler::Endian::Little, &args.input).unwrap();
    let mut emu = Emulator::new();
    emu.clear_memory();
    emu.clear_register();
    emu.load_from_u8(&bin, mips_emulator::Endian::Little)
        .unwrap();
    emu.run();
}
