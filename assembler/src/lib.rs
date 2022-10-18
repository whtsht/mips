pub mod header;
pub mod instruction;
pub mod parser;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use instruction::{allocate_data, gen_symbol_table, write_data_section};
use parser::parse;

pub type BResult<T> = Result<T, Box<dyn Error>>;
pub type Binary = i32;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Endian {
    Little,
    Big,
}

#[derive(Debug, PartialEq)]
pub enum SectionType {
    Text,
    Data,
    Word(Vec<Binary>),
    Space(Binary),
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
    Section(SectionType),
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

    // Parse input data
    let tokens = parse(&source)?;

    // Create file header
    let file_header = FileHeader::new(&tokens);

    // Gen symbol table
    let symbol_table = gen_symbol_table(&tokens, &file_header);

    // Write file header
    file_header.write_code(endian, &mut output)?;

    // Write text section
    for code in tokens.iter().filter_map(|t| t.code(&symbol_table)) {
        write_code(endian, code, &mut output)?;
    }

    // Gen global data and write data section
    let data = allocate_data(&tokens);
    write_data_section(endian, &data, &mut output)?;

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
