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

pub fn get_data_section(sections: &Vec<Vec<&Instruction>>) -> Vec<Binary> {
    let mut output = Vec::new();

    sections
        .iter()
        .filter(|s| match s[0] {
            Instruction::Section(SectionType::Data) => true,
            _ => false,
        })
        .flatten()
        .for_each(|i| match i {
            Instruction::Section(SectionType::Word(v)) => {
                output.extend(v.iter());
            }
            Instruction::Section(SectionType::Space(n)) => {
                for _ in 0..(*n / 4) {
                    output.push(0);
                }
            }
            _ => {}
        });

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

pub fn gen_symbol_table<'a>(
    sections: &'a Vec<Vec<&Instruction>>,
    file_header: &FileHeader,
) -> HashMap<&'a str, Binary> {
    let mut table = HashMap::new();

    let mut text_section = &sections[0][1..];

    for s in &sections[1..] {
        match s.get(0) {
            Some(Instruction::Section(SectionType::Text)) => text_section = &s[1..],
            _ => {}
        }
    }

    let mut count = file_header.start_text;
    for ins in text_section {
        match ins {
            Instruction::LabelDef { name } => {
                table.insert(*name, count);
            }
            _ => {
                count += 1;
            }
        }
    }

    if sections.len() < 2 {
        return table;
    }

    let mut data_section = &sections[1][1..];

    for s in &sections[1..] {
        match s.get(0) {
            Some(Instruction::Section(SectionType::Data)) => data_section = &s[1..],
            _ => {}
        }
    }

    let mut count = file_header.start_data;
    for ins in data_section {
        match ins {
            Instruction::LabelDef { name } => {
                table.insert(*name, count);
            }
            Instruction::Section(SectionType::Word(v)) => {
                count += v.len() as Binary;
            }
            Instruction::Section(SectionType::Space(n)) => {
                count += *n / 4;
            }
            _ => {
                count += 1;
            }
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
fn test_label() {
    use crate::parse;
    let input = r#"
        .text
        L1: addi $t0, $zero, L3
        L2: addi $t0, $zero, L4
        .data
        L3: .space 20
        L4: .space 16
        L5: .space 12
        "#;

    let mut tokens = parse(input).unwrap();
    if let Some(Instruction::Section(SectionType::Text)) = tokens.get(0) {
    } else {
        tokens.insert(0, Instruction::Section(SectionType::Text));
    }
    let sections = tokens.split_rinclusive(|t| match t {
        Instruction::Section(SectionType::Text) | Instruction::Section(SectionType::Data) => false,
        _ => true,
    });
    let file_header = FileHeader::new(&sections);
    let symbol_table = gen_symbol_table(&sections, &file_header);

    assert_eq!(symbol_table.get("L1"), Some(&3));
    assert_eq!(symbol_table.get("L2"), Some(&4));
    assert_eq!(symbol_table.get("L3"), Some(&5));
    assert_eq!(symbol_table.get("L4"), Some(&10));
    assert_eq!(symbol_table.get("L5"), Some(&14));
}

#[test]
fn test_data_section() {
    use crate::parse;
    let input = r#"
        .text
        L1: addi $t0, $zero, L3
        L2: addi $t0, $zero, L4
        .data
        L3: .space 20
        L4: .space 16
        L5: .word 1, 2, 3
        "#;

    let tokens = parse(input).unwrap();
    let sections = tokens.split_rinclusive(|t| match t {
        Instruction::Section(SectionType::Text) | Instruction::Section(SectionType::Data) => false,
        _ => true,
    });

    let data = get_data_section(&sections);

    assert_eq!(data, vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3]);
}

#[test]
fn test_vec() {
    let v1 = vec![0, 23, 3, 12, 0, 12, 4, 5, 0, 10, 1];

    assert_eq!(
        v1.split_rinclusive(|v| v != &0),
        vec![
            vec![&0, &23, &3, &12],
            vec![&0, &12, &4, &5],
            vec![&0, &10, &1]
        ]
    );
}

pub trait SplitRInclusive {
    type Item;
    fn split_rinclusive(&self, f: impl Fn(&Self::Item) -> bool) -> Vec<Vec<&Self::Item>>;
}

impl<T: std::fmt::Debug> SplitRInclusive for Vec<T> {
    type Item = T;

    fn split_rinclusive(&self, f: impl Fn(&Self::Item) -> bool) -> Vec<Vec<&Self::Item>> {
        let mut output = vec![vec![]];
        let mut idx = 0;

        let mut vec = self.into_iter();

        loop {
            match vec.next() {
                Some(v) if !f(&v) => {
                    output[idx].push(v);
                    break;
                }
                _ => {
                    continue;
                }
            }
        }

        for v in vec {
            if f(&v) {
                output[idx].push(v);
            } else {
                output.push(vec![]);
                idx += 1;
                output[idx].push(v);
            }
        }

        output
    }
}
