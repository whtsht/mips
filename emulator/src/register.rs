use crate::Binary;
use crate::Register;

impl Register {
    pub fn new() -> Self {
        Self { storage: [0; 34] }
    }

    pub fn get(&self, idx: Binary) -> Binary {
        match idx {
            0 => 0,
            _ => self.storage[idx as usize] as Binary,
        }
    }

    pub fn set(&mut self, idx: Binary, value: Binary) {
        match idx {
            0 => {}
            _ => {
                self.storage[idx as usize] = value;
            }
        }
    }

    pub fn reset(&mut self) {
        self.storage = [0; 34];
    }
}

#[allow(dead_code)]
impl Register {
    pub const ZERO: Binary = 0;
    pub const AT: Binary = 1;
    pub const V0: Binary = 2;
    pub const V1: Binary = 3;
    pub const A0: Binary = 4;
    pub const A1: Binary = 5;
    pub const A2: Binary = 6;
    pub const A3: Binary = 7;
    pub const GP: Binary = 28;
    pub const SP: Binary = 29;
    pub const FP: Binary = 30;
    pub const RA: Binary = 31;
    pub const HI: Binary = 32;
    pub const LO: Binary = 33;
}
