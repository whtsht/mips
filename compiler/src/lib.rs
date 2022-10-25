pub mod node;
pub mod token;

use crate::node::Node;
use anyhow::{anyhow, Result};
use std::fmt::Write;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use token::convert_error;
use token::tokenize;
use token::Punctuation;

pub fn push(output: &mut String, dest: &str) -> Result<()> {
    writeln!(output, "  addi $sp, $sp, -4")?;
    writeln!(output, "  sw {}, 4($sp)", dest)?;

    Ok(())
}

pub fn pop(output: &mut String, dest: &str) -> Result<()> {
    writeln!(output, "  lw {}, 4($sp)", dest)?;
    writeln!(output, "  addi $sp, $sp, 4")?;

    Ok(())
}

pub fn gen_code(node: Box<Node>, output: &mut String) -> Result<()> {
    match *node {
        Node::Add { lhs, rhs } => {
            gen_code(lhs, output)?;
            gen_code(rhs, output)?;
            pop(output, "$t0")?;
            pop(output, "$t1")?;
            writeln!(output, "  add $t0, $t0, $t1")?;
        }
        Node::Sub { lhs, rhs } => {
            gen_code(lhs, output)?;
            gen_code(rhs, output)?;
            pop(output, "$t0")?;
            pop(output, "$t1")?;
            writeln!(output, "  sub $t0, $t0, $t1")?;
        }
        Node::Number(n) => {
            writeln!(output, "  ori $t0, $zero, {}", n)?;
        }
    }

    push(output, "$t0")?;

    Ok(())
}

pub fn compile_from_path<P: AsRef<Path> + std::fmt::Debug>(input: P) -> Result<String> {
    let mut source = String::new();
    let mut input = File::open(input)?;

    input.read_to_string(&mut source)?;
    compile_from_string(&source)
}

pub fn compile_from_string(input: &str) -> Result<String> {
    let mut output = String::new();

    writeln!(output, ".text")?;
    writeln!(output, ".globl main")?;
    writeln!(output, "main:")?;

    let tokens = match tokenize(input) {
        Ok((_, tokens)) => tokens,
        Err((stop, message)) => Err(anyhow!("{}", convert_error(input, stop, &message)))?,
    };

    let mut tokens = tokens.iter().peekable();

    let node = match node::expr(&mut tokens) {
        Ok(node) => node,
        Err(message) => Err(anyhow!("{}", message))?,
    };

    gen_code(node, &mut output)?;

    writeln!(output, "  or $a0, $zero, $t0")?;
    writeln!(output)?;
    writeln!(output, "  ori $v0, $zero, 1")?;
    writeln!(output, "  syscall")?;
    writeln!(output, "  jr $ra")?;

    Ok(output)
}
