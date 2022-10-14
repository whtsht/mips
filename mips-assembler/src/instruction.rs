use crate::Binary;
use crate::Instruction;

impl Instruction {
    pub fn ii(op: Binary, rs: Binary, rt: Binary, im: Binary) -> Self {
        Self::I { op, rs, rt, im }
    }

    pub fn ri(op: Binary, rs: Binary, rt: Binary, rd: Binary, sh: Binary, fc: Binary) -> Self {
        Self::R {
            op,
            rs,
            rt,
            rd,
            sh,
            fc,
        }
    }

    pub fn ji(op: Binary, ad: Binary) -> Self {
        Self::J { op, ad }
    }

    pub fn code(&self) -> Binary {
        let mut code = 0;
        match self {
            Instruction::I { op, rs, rt, im } => {
                code |= op << 26;
                code |= rs << 21;
                code |= rt << 16;
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
                code |= op << 26;
                code |= rs << 21;
                code |= rt << 16;
                code |= rd << 11;
                code |= sh << 6;
                code |= fc;
            }
            Instruction::J { op, ad } => {
                code |= op << 26;
                code |= ad;
            }
        }
        code
    }
}

#[test]
fn test_instruction() {
    assert_eq!(
        Instruction::ii(0x8, 0x1, 0x2, 0xa).code(),
        0b001000_00001_00010_0000000000001010
    );
    assert_eq!(
        Instruction::ii(0x8, 0x5, 0x0, 555).code(),
        0b001000_00101_00000_0000001000101011
    );
}
