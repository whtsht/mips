use anyhow::{anyhow, Result};
use std::fmt::Write;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn integer(s: &str) -> Result<(i32, &str)> {
    let end = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    match s[..end].parse() {
        Ok(value) => Ok((value, &s[end..])),
        Err(_) => Err(anyhow!("expected number")),
    }
}
pub fn char(c: char) -> impl Fn(&str) -> Result<(char, &str)> {
    move |i: &str| {
        let mut chars = i.chars();
        if chars.next() == Some(c) {
            Ok((c, chars.as_str()))
        } else {
            Err(anyhow!("expected '{}'", c))
        }
    }
}

pub fn gen_code(mut i: &str) -> Result<String> {
    let mut output = String::new();

    writeln!(output, ".text")?;
    writeln!(output, ".globl main")?;
    writeln!(output, "main:")?;
    let (num, rest) = integer(i)?;
    i = rest;
    writeln!(output, "  ori $a0, $zero, {}", num)?;

    let add = char('+');
    let sub = char('-');

    while i.len() > 0 {
        if let Ok((_, rest)) = add(i) {
            let (num, rest) = integer(rest)?;
            writeln!(output, "  addi $a0, $a0, {}", num)?;
            i = rest;
            continue;
        }

        if let Ok((_, rest)) = sub(i) {
            let (num, rest) = integer(rest)?;
            writeln!(output, "  addi $a0, $a0, {}", -num)?;
            i = rest;
            continue;
        }

        return Err(anyhow!("unexpected string"));
    }

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

    gen_code(&source)
}
