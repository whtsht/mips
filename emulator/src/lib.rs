pub mod decode;
pub mod emu;
pub mod register;

pub type Binary = i32;
const MEMORY_SIZE: usize = 8192;

pub struct Register {
    storage: [Binary; 34],
}

pub enum Endian {
    Little,
    Big,
}

pub fn as_i32_be(array: &[u8]) -> i32 {
    ((array[0] as i32) << 24)
        + ((array[1] as i32) << 16)
        + ((array[2] as i32) << 8)
        + ((array[3] as i32) << 0)
}

pub fn as_i32_le(array: &[u8]) -> i32 {
    ((array[0] as i32) << 0)
        + ((array[1] as i32) << 8)
        + ((array[2] as i32) << 16)
        + ((array[3] as i32) << 24)
}

pub struct Emulator {
    pub register: Register,
    pub memory: Vec<Binary>,
    pub pc: Binary,
    pub stdout_history: String,
}
