use crate::write_code;
use crate::BResult;
use crate::Binary;
use crate::Endian;
use crate::FileHeader;
use crate::Instruction;
use crate::SectionType;

impl FileHeader {
    const HEADER_SIZE: Binary = 3;

    pub fn new(sections: &Vec<Vec<&Instruction>>) -> Self {
        let entry_point = FileHeader::HEADER_SIZE;
        let start_text = FileHeader::HEADER_SIZE;

        let mut text_section = &sections[0];

        for s in &sections[1..] {
            match s.get(0) {
                Some(Instruction::Section(SectionType::Text)) => text_section = s,
                _ => {}
            }
        }

        let start_data = text_section
            .iter()
            .filter(|v| match v {
                Instruction::I { .. } | Instruction::R { .. } | Instruction::J { .. } => true,
                _ => false,
            })
            .count() as Binary
            + start_text;

        Self {
            entry_point,
            start_text,
            start_data,
        }
    }

    pub fn write_code(&self, endian: Endian, output: &mut Vec<u8>) -> BResult<()> {
        write_code(endian, self.entry_point, output)?;
        write_code(endian, self.start_text, output)?;
        write_code(endian, self.start_data, output)?;
        Ok(())
    }
}
