use crate::as_i32_be;
use crate::as_i32_le;
use crate::decode::{II, RI};
use crate::Binary;
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
            pc: 0,
        }
    }

    pub fn clear_memory(&mut self) {
        self.memory.iter_mut().for_each(|m| *m = 0);
    }

    pub fn load_program<P: AsRef<Path>>(
        &mut self,
        input: P,
        endian: Endian,
    ) -> Result<(), std::io::Error> {
        let mut input = File::open(input)?;
        let mut buffer = vec![];

        input.read_to_end(&mut buffer)?;

        for (idx, buf) in buffer.chunks(4).enumerate() {
            match endian {
                Endian::Little => self.memory[idx] = as_i32_le(buf),
                Endian::Big => self.memory[idx] = as_i32_be(buf),
            }
        }

        // Entry point
        self.pc = self.memory[0];

        Ok(())
    }

    pub fn init(&mut self) {
        self.pc = 0;
        self.register.reset();
    }

    pub fn syscall(&mut self) {
        let v0 = self.register.get(Register::V0);
        if v0 == 1 {
            let a0 = self.register.get(Register::A0);
            println!("{}", a0);
        }
    }

    pub fn step(&mut self) {
        let code = self.memory[self.pc as usize];
        if code >> 26 == 0x0 {
            let ri = RI::decode(code);
            if ri.fc == 0x8 {
                self.pc = self.register.get(ri.rs);
                return;
            }

            if ri.fc == 0xc {
                self.syscall();
                self.pc += 1;
                return;
            }

            let ri = RI::decode(code);
            if arithmetic_with_register(&mut self.register, &ri) {
                self.pc += 1;
                return;
            }
        }

        if arithmetic_with_immediate(&mut self.register, code) {
            self.pc += 1;
            return;
        }

        println!("failed to decode a instruction [PC = {}]", self.pc);
        std::process::exit(1);
    }

    pub fn run(&mut self) {
        self.init();

        loop {
            self.step();

            if self.pc == 0 {
                break;
            }
        }
    }

    pub fn info_register(&self) {
        for (i, r) in self.register.storage.iter().enumerate() {
            println!("${}: {:#x} | {}", i, r, r);
        }
    }
}

pub fn arithmetic_with_register(register: &mut Register, i: &RI) -> bool {
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
        _ => false,
    }
}

pub fn arithmetic_with_immediate(register: &mut Register, code: Binary) -> bool {
    match code >> 26 {
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
    match code >> 26 {
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
