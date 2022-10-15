use mips_assembler::assemble_to_u8;
use mips_emu::Emulator;

fn assert(fname: &str, expect: &str) {
    let bin = assemble_to_u8(mips_assembler::Endian::Little, fname).unwrap();
    let mut emu = Emulator::new();
    emu.clear_memory();
    emu.clear_register();
    emu.load_from_u8(&bin, mips_emu::Endian::Little).unwrap();
    emu.run();
    assert_eq!(emu.stdout_history, expect);
}

#[test]
fn test() {
    assert("./tests/case1.s", "5");
    assert("./tests/case2.s", "-34");
    assert("./tests/case3.s", "0123456789");
}
