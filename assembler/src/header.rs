use crate::write_code;
use crate::BResult;
use crate::Binary;
use crate::Endian;
use crate::FileHeader;
use crate::Instruction;

impl FileHeader {
    const HEADER_SIZE: Binary = 3;

    pub fn new(tokens: &Vec<Instruction>) -> Self {
        let entry_point = FileHeader::HEADER_SIZE;
        let start_text = FileHeader::HEADER_SIZE;
        let start_data = tokens
            .iter()
            .filter(|t| match t {
                Instruction::I { .. } | Instruction::R { .. } | Instruction::J { .. } => true,
                _ => false,
            })
            .count() as Binary
            + FileHeader::HEADER_SIZE;
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
