use mips_assembler::assemble_to_u8;
use mips_emulator::Emulator;

fn assert(fname: &str, expect: &str) {
    println!("Start assemble");
    let fname = format!("./tests/assemble_to_emulate/{}", fname);
    let bin = assemble_to_u8(mips_assembler::Endian::Little, &fname).unwrap();
    println!("Finish assemble");
    let mut emu = Emulator::new();
    emu.clear_memory();
    emu.clear_register();
    emu.load_from_u8(&bin, mips_emulator::Endian::Little)
        .unwrap();
    emu.run();
    assert_eq!(emu.stdout_history, expect);
    println!("Finish emulate");
}

#[test]
fn test() {
    assert("001_addi_addu.s", "5");
    assert("002_label.s", "-34");
    assert("003_loop.s", "0123456789");
    assert("004_data.s", "100");
    assert("005_slt.s", "0123456789");
    assert("006_mul.s", "20");
    assert("007_div.s", "62");
    assert("008_shift.s", "-32-2");
    assert("009_space.s", "15");
    assert("010_ori.s", "-10");
}
