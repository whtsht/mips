pub mod instruction;
pub mod parser;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use instruction::gen_symbol_table;
use parser::parse;

pub type BResult<T> = Result<T, Box<dyn Error>>;
pub type Binary = i32;

pub enum Endian {
    Little,
    Big,
}

#[derive(Debug, PartialEq)]
pub enum Operand<'a> {
    Register(Binary),
    Label(&'a str),
}

#[derive(Debug, PartialEq)]
pub struct Operation(Binary);

#[derive(Debug, PartialEq)]
pub enum Instruction<'a> {
    I {
        op: Operation,
        rs: Operand<'a>,
        rt: Operand<'a>,
        im: Binary,
    },
    R {
        op: Operation,
        rs: Operand<'a>,
        rt: Operand<'a>,
        rd: Operand<'a>,
        sh: Binary,
        fc: Binary,
    },
    J {
        op: Operation,
        ad: Operand<'a>,
    },
    LabelDef {
        name: &'a str,
    },
}

pub fn assemble_to_u8<P: AsRef<Path> + std::fmt::Display>(
    endian: Endian,
    input: P,
) -> BResult<Vec<u8>> {
    let mut output = Vec::new();
    let mut source = String::new();
    let mut input = File::open(input)?;
    input.read_to_string(&mut source)?;

    let tokens = parse(&source)?;
    let symbol_table = gen_symbol_table(&tokens);

    for code in tokens.iter().filter_map(|t| t.code(&symbol_table)) {
        match endian {
            Endian::Big => output.write_all(&code.to_be_bytes())?,
            Endian::Little => output.write_all(&code.to_le_bytes())?,
        }
    }
    Ok(output)
}

pub fn assemble<P: AsRef<Path> + std::fmt::Display>(
    endian: Endian,
    input: P,
    output: P,
) -> BResult<()> {
    let mut input =
        File::open(&input).or_else(|_| Err(format!("A file named {} was not found", input)))?;
    let mut output = File::create(output)?;
    let mut source = String::new();
    input.read_to_string(&mut source)?;

    let tokens = parse(&source)?;
    let symbol_table = gen_symbol_table(&tokens);

    for code in tokens.iter().filter_map(|t| t.code(&symbol_table)) {
        match endian {
            Endian::Big => output.write_all(&code.to_be_bytes())?,
            Endian::Little => output.write_all(&code.to_le_bytes())?,
        }
    }

    Ok(())
}
