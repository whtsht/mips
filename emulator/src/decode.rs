use crate::Binary;

pub struct Mask {}
#[allow(dead_code, overflowing_literals)]
impl Mask {
    const OP: Binary = 0b111111_00000_00000_00000_00000_000000;
    const RS: Binary = 0b000000_11111_00000_00000_00000_000000;
    const RT: Binary = 0b000000_00000_11111_00000_00000_000000;
    const RD: Binary = 0b000000_00000_00000_11111_00000_000000;
    const SH: Binary = 0b000000_00000_00000_00000_11111_000000;
    const FC: Binary = 0b000000_00000_00000_00000_00000_111111;
    const SI: Binary = 0b000000_00000_00000_10000_00000_000000;
}

#[derive(Debug, PartialEq)]
pub struct II {
    pub rs: Binary,
    pub rt: Binary,
    pub im: Binary,
}

#[derive(Debug, PartialEq)]
pub struct JI {
    pub ad: Binary,
}

#[derive(Debug, PartialEq)]
pub struct RI {
    pub rs: Binary,
    pub rt: Binary,
    pub rd: Binary,
    pub sh: Binary,
    pub fc: Binary,
}

impl II {
    pub fn decode(code: Binary) -> II {
        let rs = (code & !Mask::OP) >> 21;
        let rt = (code & !(Mask::OP | Mask::RS)) >> 16;
        let im = code & !(Mask::OP | Mask::RS | Mask::RT) | ((1 << 16) - ((code >> 15) & 1)) << 16;
        II { rs, rt, im }
    }
}

impl RI {
    pub fn decode(code: Binary) -> RI {
        let rs = (code & !Mask::OP) >> 21;
        let rt = (code & !(Mask::OP | Mask::RS)) >> 16;
        let rd = (code & !(Mask::OP | Mask::RS | Mask::RT)) >> 11;
        let sh = (code & !(Mask::OP | Mask::RS | Mask::RT | Mask::RD)) >> 6;
        let fc = code & !(Mask::OP | Mask::RS | Mask::RT | Mask::RD | Mask::SH);
        RI { rs, rt, rd, sh, fc }
    }
}

impl JI {
    pub fn decode(code: Binary) -> JI {
        let ad = code & !Mask::OP;
        JI { ad }
    }
}

#[test]
#[allow(overflowing_literals)]
fn test_ii_decode() {
    let ii = II::decode(0b001000_00001_00010_0000000000001010);
    assert_eq!(
        ii,
        II {
            rs: 1,
            rt: 2,
            im: 10
        }
    );

    let ii = II::decode(0b001000_00001_00010_1111111111110110);
    assert_eq!(
        ii,
        II {
            rs: 1,
            rt: 2,
            im: -10
        }
    );
}

#[test]
fn test_ri_decode() {
    let ri = RI::decode(0b000000_00000_00000_00000_00000_001000);
    assert_eq!(
        ri,
        RI {
            rs: 0,
            rt: 0,
            rd: 0,
            sh: 0,
            fc: 8,
        }
    );
}

#[test]
fn test_ji_decode() {
    let ji = JI::decode(0b000000_00000_00000_0000000001111111);
    assert_eq!(ji, JI { ad: 127 });
}
