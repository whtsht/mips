use crate::write_code;
use crate::BResult;
use crate::Endian;
use crate::FileHeader;
use std::collections::HashMap;

use crate::Binary;
use crate::Instruction;
use crate::Operand;
use crate::Operation;
use crate::SectionType;

impl<'a> Operand<'a> {
    fn to_binary(&self, symbol_table: &HashMap<&str, Binary>) -> Binary {
        match self {
            Operand::Register(b) => *b,
            Operand::Label(name) => {
                if let Some(b) = symbol_table.get(name) {
                    *b
                } else {
                    panic!("Label {} is not defined", name);
                }
            }
            Operand::Constant(b) => *b,
        }
    }
}

impl Operation {
    fn to_binary(&self) -> Binary {
        self.0
    }
}

pub fn allocate_data(tokens: &Vec<Instruction>) -> Vec<Binary> {
    let mut output = Vec::new();
    for token in tokens.iter() {
        match token {
            Instruction::Section(SectionType::Word(w)) => output.extend(w.iter()),
            Instruction::Section(SectionType::Space(n)) => {
                for _ in 0..(*n / 4) {
                    output.push(0);
                }
            }
            _ => {}
        }
    }
    output
}

pub fn write_data_section(
    endian: Endian,
    codes: &Vec<Binary>,
    output: &mut Vec<u8>,
) -> BResult<()> {
    for code in codes.iter() {
        write_code(endian, *code, output)?;
    }
    Ok(())
}

#[test]
fn test_allocate_data() {
    let tokens = vec![
        Instruction::Section(SectionType::Word(vec![1, 2, 3])),
        Instruction::Section(SectionType::Word(vec![100, 200, 300])),
        Instruction::Section(SectionType::Space(12)),
    ];
    let b = allocate_data(&tokens);
    assert_eq!(b, vec![1, 2, 3, 100, 200, 300, 0, 0, 0])
}

pub fn gen_symbol_table<'a>(
    tokens: &'a Vec<Instruction>,
    file_header: &FileHeader,
) -> HashMap<&'a str, Binary> {
    let mut table = HashMap::new();
    let mut tokens = tokens.iter();

    for (count, token) in tokens
        .by_ref()
        .take_while(|t| match t {
            Instruction::Section(_) => false,
            _ => true,
        })
        .enumerate()
    {
        match token {
            Instruction::LabelDef { name } => {
                if table.get(name).is_some() {
                    panic!("Label {} is duplicate", name);
                }
                table.insert(*name, count as Binary + file_header.start_text);
            }
            _ => {}
        }
    }

    for (count, token) in tokens.enumerate() {
        match token {
            Instruction::LabelDef { name } => {
                if table.get(name).is_some() {
                    panic!("Label {} is duplicate", name);
                }
                table.insert(*name, count as Binary + file_header.start_data);
            }
            _ => {}
        }
    }

    table
}

impl<'a> Instruction<'a> {
    pub fn ii(op: Operation, rs: Operand<'a>, rt: Operand<'a>, im: Operand<'a>) -> Self {
        Self::I { op, rs, rt, im }
    }

    pub fn ri(
        op: Operation,
        rs: Operand<'a>,
        rt: Operand<'a>,
        rd: Operand<'a>,
        sh: Operand<'a>,
        fc: Operand<'a>,
    ) -> Self {
        Self::R {
            op,
            rs,
            rt,
            rd,
            sh,
            fc,
        }
    }

    pub fn ji(op: Operation, ad: Operand<'a>) -> Self {
        Self::J { op, ad }
    }

    pub fn code(&self, symbol_table: &HashMap<&str, Binary>) -> Option<Binary> {
        let mut code = 0;
        match self {
            Instruction::I { op, rs, rt, im } => {
                code |= op.to_binary() << 26;
                code |= rs.to_binary(symbol_table) << 21;
                code |= rt.to_binary(symbol_table) << 16;
                code |= 0b000000_00000_00000_11111_11111_111111 & im.to_binary(symbol_table);
            }
            Instruction::R {
                op,
                rs,
                rt,
                rd,
                sh,
                fc,
            } => {
                code |= op.to_binary() << 26;
                code |= rs.to_binary(symbol_table) << 21;
                code |= rt.to_binary(symbol_table) << 16;
                code |= rd.to_binary(symbol_table) << 11;
                code |= sh.to_binary(symbol_table) << 6;
                code |= fc.to_binary(symbol_table);
            }
            Instruction::J { op, ad } => {
                code |= op.to_binary() << 26;
                code |= ad.to_binary(symbol_table);
            }
            Instruction::LabelDef { .. } => return None,
            Instruction::Section(_) => return None,
        }
        Some(code)
    }
}

#[test]
fn test_instruction() {
    // assert_eq!(
    //     Token::ii(0x8, 0x1, 0x2, 0xa).code(),
    //     0b001000_00001_00010_0000000000001010
    // );
    // assert_eq!(
    //     Token::ii(0x8, 0x5, 0x0, 555).code(),
    //     0b001000_00101_00000_0000001000101011
    // );
}
