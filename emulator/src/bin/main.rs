use mips_emulator::{Emulator, Endian};

fn main() {
    let mut args = std::env::args();
    args.next();

    if let Some(input) = args.next() {
        let mut emu = Emulator::new();

        emu.load_program(input, Endian::Little)
            .expect("failed to load file");

        emu.run();

        let exit_code = emu.register.get(1) as i32;

        std::process::exit(exit_code);
    } else {
        println!("expected file name");
    }
}
