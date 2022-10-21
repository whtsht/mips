use anyhow::Result;
use std::fmt::Write;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn gen_code(input: String) -> Result<String> {
    let mut output = String::new();

    writeln!(output, ".text")?;
    writeln!(output, ".globl main")?;
    writeln!(output, "main:")?;
    writeln!(output, "  ori $a0, $zero, {}", input.parse::<i32>()?)?;
    writeln!(output, "  ori $v0, $zero, 1")?;
    writeln!(output, "  syscall")?;
    writeln!(output, "  jr $ra")?;

    Ok(output)
}

pub fn compile_from_path<P: AsRef<Path> + std::fmt::Debug>(input: P) -> Result<String> {
    let mut source = String::new();
    let mut input = File::open(input)?;

    input.read_to_string(&mut source)?;
    source = source.replace("\n", "");

    gen_code(source)
}
