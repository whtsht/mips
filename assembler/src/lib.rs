pub mod header;
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Endian {
    Little,
    Big,
}

#[derive(Debug, PartialEq)]
pub struct FileHeader {
    entry_point: Binary,
    start_text: Binary,
    start_data: Binary,
}

#[derive(Debug, PartialEq)]
pub enum Operand<'a> {
    Register(Binary),
    Label(&'a str),
    Constant(Binary),
}

#[derive(Debug, PartialEq)]
pub struct Operation(Binary);

#[derive(Debug, PartialEq)]
pub enum Presudo {
    Word(Vec<Binary>),
}

#[derive(Debug, PartialEq)]
pub enum Instruction<'a> {
    I {
        op: Operation,
        rs: Operand<'a>,
        rt: Operand<'a>,
        im: Operand<'a>,
    },
    R {
        op: Operation,
        rs: Operand<'a>,
        rt: Operand<'a>,
        rd: Operand<'a>,
        sh: Operand<'a>,
        fc: Operand<'a>,
    },
    J {
        op: Operation,
        ad: Operand<'a>,
    },
    LabelDef {
        name: &'a str,
    },
}

fn write_code(endian: Endian, code: Binary, output: &mut Vec<u8>) -> BResult<()> {
    match endian {
        Endian::Big => output.write_all(&code.to_be_bytes())?,
        Endian::Little => output.write_all(&code.to_le_bytes())?,
    }
    Ok(())
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
    let file_header = FileHeader::new(&tokens);

    file_header.write_code(endian, &mut output)?;

    for code in tokens
        .iter()
        .filter_map(|t| t.code(&symbol_table, &file_header))
    {
        write_code(endian, code, &mut output)?;
    }
    Ok(output)
}

pub fn assemble<P: AsRef<Path> + std::fmt::Display>(
    endian: Endian,
    input: P,
    output: P,
) -> BResult<()> {
    let data = assemble_to_u8(endian, input)?;
    let mut output = File::create(output)?;
    output.write_all(&data)?;

    Ok(())
}
