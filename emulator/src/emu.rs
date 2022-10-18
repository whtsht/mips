use crate::as_i32_be;
use crate::as_i32_le;
use crate::decode::JI;
use crate::decode::{II, RI};
use crate::Binary;
use crate::EBinary;
use crate::Emulator;
use crate::Endian;
use crate::Register;
use crate::MEMORY_SIZE;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

impl Emulator {
    pub fn new() -> Self {
        Self {
            register: Register::new(),
            memory: vec![0; MEMORY_SIZE],
            stdout_history: String::new(),
            pc: 0,
        }
    }

    pub fn load_program<P: AsRef<Path>>(
        &mut self,
        input: P,
        endian: Endian,
    ) -> Result<(), std::io::Error> {
        let mut input = File::open(input)?;
        let mut buffer = vec![];

        input.read_to_end(&mut buffer)?;

        self.load_from_u8(&buffer, endian)?;

        Ok(())
    }

    pub fn load_from_u8(&mut self, input: &Vec<u8>, endian: Endian) -> Result<(), std::io::Error> {
        for (idx, buf) in input.chunks(4).enumerate() {
            match endian {
                Endian::Little => self.memory[idx] = as_i32_le(buf),
                Endian::Big => self.memory[idx] = as_i32_be(buf),
            }
        }

        // Entry point
        self.pc = self.memory[0];

        Ok(())
    }

    pub fn clear_memory(&mut self) {
        self.memory.iter_mut().for_each(|m| *m = 0);
    }

    pub fn clear_register(&mut self) {
        self.register.reset();
    }

    pub fn syscall(&mut self) -> bool {
        let v0 = self.register.get(Register::V0);
        if v0 == 1 {
            let a0 = self.register.get(Register::A0);

            print!("{}", a0);
            std::io::stdout().flush().unwrap();
            self.stdout_history.push_str(&format!("{}", a0));
            return true;
        }

        false
    }

    pub fn step(&mut self) {
        let code = self.memory[self.pc as usize];

        // For debug
        // println!("{}: {:032b}", self.pc, code);

        if let Some(jd) = branch_instruction(&mut self.register, code) {
            match jd {
                JumpDest::Spec(pc) => self.pc = pc,
                JumpDest::Next => self.pc += 1,
            }
            return;
        }

        if let Some(pc) = jump_instruction(&mut self.register, code) {
            self.pc = pc;
            return;
        }

        if funct(code) == 0xc && self.syscall() {
            self.pc += 1;
            return;
        }

        if arithmetic_with_register(&mut self.register, code) {
            self.pc += 1;
            return;
        }

        if arithmetic_with_immediate(&mut self.register, code) {
            self.pc += 1;
            return;
        }

        if memory_instruction(&mut self.register, &mut self.memory, code) {
            self.pc += 1;
            return;
        }

        if move_from(&mut self.register, code) {
            self.pc += 1;
            return;
        }

        panic!("failed to decode a instruction [PC = {}]", self.pc);
    }

    pub fn run(&mut self) {
        loop {
            self.step();
            if self.pc == 0 {
                return;
            }
        }
    }

    pub fn info_register(&self) {
        for (i, r) in self.register.storage.iter().enumerate() {
            println!("${}: {:#x} | {}", i, r, r);
        }
    }
}

pub fn opcode(code: Binary) -> Binary {
    (code as u32 >> 26) as Binary
}

pub fn funct(code: Binary) -> Binary {
    code & 0b000000_00000_00000_00000_00000_111111
}

pub enum JumpDest {
    Next,
    Spec(Binary),
}

pub fn jump_instruction(register: &mut Register, code: Binary) -> Option<Binary> {
    let opcode = opcode(code);
    // Jump Register
    if opcode == 0x0 {
        let ri = RI::decode(code);

        if ri.fc == 0x8 {
            let pc = register.get(ri.rs);
            return Some(pc);
        }
    // Jump
    } else if opcode == 0x2 {
        let ji = JI::decode(code);
        return Some(ji.ad);
    }
    None
}

fn move_from(register: &mut Register, code: Binary) -> bool {
    if opcode(code) != 0x0 {
        return false;
    }
    let i = RI::decode(code);

    match i.fc {
        0x10 => {
            let hi = register.get(Register::HI);
            register.set(i.rd, hi);
            true
        }
        0x12 => {
            let lo = register.get(Register::LO);
            register.set(i.rd, lo);
            true
        }
        _ => false,
    }
}

fn branch_instruction(register: &mut Register, code: Binary) -> Option<JumpDest> {
    match opcode(code) {
        // Branch On Equal
        0x4 => {
            let ii = II::decode(code);

            if register.get(ii.rs) == register.get(ii.rt) {
                return Some(JumpDest::Spec(ii.im));
            } else {
                return Some(JumpDest::Next);
            }
        }
        // Branch On Not Equal
        0x5 => {
            let ii = II::decode(code);

            if register.get(ii.rs) != register.get(ii.rt) {
                return Some(JumpDest::Spec(ii.im));
            } else {
                return Some(JumpDest::Next);
            }
        }
        _ => None,
    }
}

pub fn arithmetic_with_register(register: &mut Register, code: Binary) -> bool {
    if opcode(code) != 0x0 {
        return false;
    }
    let i = RI::decode(code);
    match i.fc {
        // Add Unsigned
        0x21 => {
            let rs = register.get(i.rs);
            let rt = register.get(i.rt);
            register.set(i.rd, rs.wrapping_add(rt));
            true
        }
        // Sub Unsigned
        0x23 => {
            let rs = register.get(i.rs);
            let rt = register.get(i.rt);
            register.set(i.rd, rs.wrapping_sub(rt));
            true
        }
        // And
        0x24 => {
            let rs = register.get(i.rs);
            let rt = register.get(i.rt);
            register.set(i.rd, rs & rt);
            true
        }
        // Or
        0x25 => {
            let rs = register.get(i.rs);
            let rt = register.get(i.rt);
            register.set(i.rd, rs | rt);
            true
        }
        // Set Less Than
        0x2a => {
            if register.get(i.rs) < register.get(i.rt) {
                register.set(i.rd, 1);
            } else {
                register.set(i.rd, 0);
            }
            true
        }
        // Divide
        0x1a => {
            let rs = register.get(i.rs);
            let rt = register.get(i.rt);
            let lo = rs / rt;
            let hi = rs % rt;

            register.set(Register::HI, hi);
            register.set(Register::LO, lo);
            true
        }
        // Divide Unsigned
        0x1b => {
            let rs = register.get(i.rs);
            let rt = register.get(i.rt);
            let lo = rs / rt;
            let hi = rs % rt;

            register.set(Register::HI, hi);
            register.set(Register::LO, lo);

            true
        }
        // Multiply
        0x18 => {
            let rs = register.get(i.rs) as EBinary;
            let rt = register.get(i.rt) as EBinary;

            let rd = rs * rt;

            let lm = ((1 as EBinary) << 32) - 1;
            let hm = (((1 as EBinary) << 32) - 1) << 32;

            let lo = rd & lm;
            let hi = (rd & hm) >> 32;

            register.set(Register::HI, hi as Binary);
            register.set(Register::LO, lo as Binary);
            true
        }
        // Multiply Unsigned
        0x19 => {
            let rs = register.get(i.rs) as EBinary;
            let rt = register.get(i.rt) as EBinary;

            let rd = rs * rt;

            let lm = ((1 as EBinary) << 32) - 1;
            let hm = (((1 as EBinary) << 32) - 1) << 32;

            let lo = rd & lm;
            let hi = (rd & hm) >> 32;

            register.set(Register::HI, hi as Binary);
            register.set(Register::LO, lo as Binary);
            true
        }
        _ => false,
    }
}

pub fn arithmetic_with_immediate(register: &mut Register, code: Binary) -> bool {
    match opcode(code) {
        // Add Immediate
        0x8 => {
            let ii = II::decode(code);
            let rs = register.get(ii.rs);
            let im = ii.im;

            register.set(ii.rt, rs.wrapping_add(im));
            true
        }
        // Add Immediate Unsigned
        0x9 => {
            let ii = II::decode(code);
            let rs = register.get(ii.rs);
            let im = ii.im;

            register.set(ii.rt, rs.wrapping_add(im));
            true
        }
        _ => false,
    }
}

pub fn memory_instruction(register: &mut Register, memory: &mut [Binary], code: Binary) -> bool {
    match opcode(code) {
        // Load Word
        0x23 => {
            let ii = II::decode(code);
            let rs = register.get(ii.rs);
            let im = ii.im / 4;
            let s = memory[(rs + im) as usize];

            register.set(ii.rt, s);
            true
        }
        // Store Word
        0x2b => {
            let ii = II::decode(code);
            let rs = register.get(ii.rs);
            let im = ii.im / 4;
            memory[(rs + im) as usize] = register.get(ii.rt);

            true
        }
        _ => false,
    }
}
