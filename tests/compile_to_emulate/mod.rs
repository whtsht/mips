use mips_assembler::assemble_to_u8_from_string;
use mips_compiler::compile_from_path;
use mips_emulator::Emulator;

fn assert(fname: &str, expect: &str) {
    println!("Start compile");
    let input = format!("./tests/compile_to_emulate/{}", fname);
    let input = compile_from_path(input).unwrap();

    println!("End compile\nStart assemble");
    let bin = assemble_to_u8_from_string(mips_assembler::Endian::Little, input).unwrap();

    println!("Finish assemble\nStart emulate");
    let mut emu = Emulator::new();
    emu.clear_memory();
    emu.clear_register();
    emu.load_from_u8(&bin, mips_emulator::Endian::Little)
        .unwrap();
    emu.run();
    println!("Finish emulate");

    assert_eq!(emu.stdout_history, expect);
}

#[test]
fn test() {
    assert("001_single_integer.lzy", "42");
    assert("002_add_sub.lzy", "21");
    assert("003_tokenize.lzy", "41");
}
