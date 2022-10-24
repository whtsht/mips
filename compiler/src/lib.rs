pub mod token;

use anyhow::{anyhow, Result};
use std::fmt::Write;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use token::tokenize;
use token::Punctuation;

pub fn gen_code(i: &str) -> Result<String> {
    let mut output = String::new();

    writeln!(output, ".text")?;
    writeln!(output, ".globl main")?;
    writeln!(output, "main:")?;

    let tokens = tokenize(i).unwrap().1;

    let mut tokens = tokens.iter().peekable();

    writeln!(output, "  ori $a0, $zero, {}", expect_number(&mut tokens)?)?;

    while tokens.len() > 0 {
        if consume(&mut tokens, Punctuation::Plus) {
            writeln!(output, "  addi $a0, $a0, {}", expect_number(&mut tokens)?)?;
            continue;
        }

        if consume(&mut tokens, Punctuation::Minus) {
            writeln!(output, "  addi $a0, $a0, -{}", expect_number(&mut tokens)?)?;
            continue;
        }

        return Err(anyhow!("unexpected string"));
    }

    writeln!(output)?;
    writeln!(output, "  ori $v0, $zero, 1")?;
    writeln!(output, "  syscall")?;
    writeln!(output, "  jr $ra")?;

    Ok(output)
}

pub fn compile_from_path<P: AsRef<Path> + std::fmt::Debug>(input: P) -> Result<String> {
    let mut source = String::new();
    let mut input = File::open(input)?;

    input.read_to_string(&mut source)?;

    gen_code(&source)
}
