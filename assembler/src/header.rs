use crate::write_code;
use crate::BResult;
use crate::Binary;
use crate::Endian;
use crate::FileHeader;
use crate::Instruction;

impl FileHeader {
    pub fn new(tokens: &Vec<Instruction>) -> Self {
        let entry_point = 3;
        let start_text = 3;
        let start_data = tokens
            .iter()
            .filter(|t| match t {
                Instruction::I { .. } | Instruction::R { .. } | Instruction::J { .. } => true,
                _ => false,
            })
            .count() as Binary
            + 3;
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
