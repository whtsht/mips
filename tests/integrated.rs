use mips_assembler::assemble_to_u8;
use mips_emu::Emulator;
use std::fs;

fn assert(fname: &str) {
    let bin = assemble_to_u8(mips_assembler::Endian::Little, fname).unwrap();
    let mut emu = Emulator::new();
    emu.load_from_u8(&bin, mips_emu::Endian::Little).unwrap();
    emu.run();
    assert_eq!(emu.stdout_history, String::from("5"));
}

#[test]
fn test() -> std::io::Result<()> {
    let paths = fs::read_dir("./tests")?;

    for path in paths {
        let fname = path?.path().display().to_string();
        if fname == "./tests/integrated.rs" {
            continue;
        }
        assert(&fname);
    }

    Ok(())
}
