use std::collections::HashMap;

use crate::Binary;
use crate::Instruction;
use crate::Operand;
use crate::Operation;

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
        }
    }
}

impl Operation {
    fn to_binary(&self) -> Binary {
        self.0
    }
}

pub fn gen_symbol_table<'a>(tokens: &'a Vec<Instruction>) -> HashMap<&'a str, Binary> {
    let mut table = HashMap::new();
    for (count, token) in tokens.iter().enumerate() {
        match token {
            Instruction::LabelDef { name } => {
                if table.get(name).is_some() {
                    panic!("Label {} is duplicate", name);
                }
                table.insert(*name, count as Binary);
            }
            _ => {}
        }
    }
    table
}

impl<'a> Instruction<'a> {
    pub fn ii(op: Operation, rs: Operand<'a>, rt: Operand<'a>, im: Binary) -> Self {
        Self::I { op, rs, rt, im }
    }

    pub fn ri(
        op: Operation,
        rs: Operand<'a>,
        rt: Operand<'a>,
        rd: Operand<'a>,
        sh: Binary,
        fc: Binary,
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
                code |= 0b000000_00000_00000_11111_11111_111111 & im;
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
                code |= sh << 6;
                code |= fc;
            }
            Instruction::J { op, ad } => {
                code |= op.to_binary() << 26;
                code |= ad.to_binary(symbol_table);
            }
            Instruction::LabelDef { name: _ } => return None,
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
