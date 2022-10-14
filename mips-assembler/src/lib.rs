pub mod instruction;
pub mod parser;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use parser::parse;

pub type AResult<T> = Result<T, Box<dyn Error>>;
pub type Binary = i32;

pub enum Endian {
    Little,
    Big,
}

#[derive(Debug, PartialEq)]
pub enum Instruction {
    I {
        op: Binary,
        rs: Binary,
        rt: Binary,
        im: Binary,
    },
    R {
        op: Binary,
        rs: Binary,
        rt: Binary,
        rd: Binary,
        sh: Binary,
        fc: Binary,
    },
    J {
        op: Binary,
        ad: Binary,
    },
}

pub fn assemble<P: AsRef<Path> + std::fmt::Display>(
    endian: Endian,
    input: P,
    output: P,
) -> AResult<()> {
    let mut input =
        File::open(&input).or_else(|_| Err(format!("A file named {} was not found", input)))?;
    let mut output = File::create(output)?;
    let mut source = String::new();
    input.read_to_string(&mut source)?;

    let tokens = parse(&source);

    for code in tokens.iter().map(|t| t.code()) {
        match endian {
            Endian::Big => output.write_all(&code.to_be_bytes())?,
            Endian::Little => output.write_all(&code.to_le_bytes())?,
        }
    }

    Ok(())
}
