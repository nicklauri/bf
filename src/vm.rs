use std::{
    array,
    io::{stdin, stdout, Read, Stdout, StdoutLock, Write},
};

use anyhow::{bail, Result};

use crate::{
    lexer,
    opcodes::{OpCode, OpCodeType},
    parser::{self, Program},
};

pub const DEFAULT_VM_MEM_SIZE: usize = 30_000;

#[derive(Debug)]
pub struct Vm {
    program: Vec<OpCode>,
    pc: usize,
    mem: Vec<u8>,
    mem_ptr: usize,
}

impl Vm {
    pub fn new(src: &str) -> Result<Self> {
        let tokens = lexer::parse(src);

        parser::parse(tokens).and_then(Self::from_program)
    }

    pub fn from_program(program: Vec<OpCode>) -> Result<Self> {
        let vm = Self {
            program,
            pc: 0,
            mem: vec![0; DEFAULT_VM_MEM_SIZE],
            mem_ptr: 0,
        };

        vm.verify_program()?;

        Ok(vm)
    }

    pub fn program(&self) -> &Program {
        &self.program
    }

    pub fn verify_program(&self) -> Result<()> {
        // TODO: verify program
        //  - correct jump?
        //  - data for Add and Sub instruction must less than u8::MAX
        let iter = self.program.iter().map(OpCode::to_tuple);
        for (inst, data) in iter {
            match inst {
                OpCodeType::Add | OpCodeType::Sub => {
                    if data >= u8::MAX as _ {
                        bail!("Add and Sub instruction must have data less than or equal to u8::MAX, data={}", data)
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    #[inline]
    pub fn get_cell(&self) -> u8 {
        // SAFETY:
        //      self.mem_ptr is already checked in self.shift_right.
        //      self.mem_ptr can only be increased in self.shift_right.
        //      self.shift_left decreases self.mem_ptr but with smallest number is 0.
        unsafe { *self.mem.get_unchecked(self.mem_ptr) }
    }

    #[inline]
    pub fn get_cell_mut(&mut self) -> &mut u8 {
        // SAFETY:
        //      self.mem_ptr is already checked in self.shift_right.
        //      self.mem_ptr can only be increased in self.shift_right.
        //      self.shift_left decreases self.mem_ptr but with smallest number is 0.
        unsafe { self.mem.get_unchecked_mut(self.mem_ptr) }
    }

    #[inline]
    pub fn add_to_cell(&mut self, amount: usize) {
        let cell = self.get_cell_mut();
        *cell = cell.wrapping_add(amount as u8);
    }

    #[inline]
    pub fn sub_to_cell(&mut self, amount: usize) {
        let cell = self.get_cell_mut();
        *cell = cell.wrapping_sub(amount as u8);
    }

    #[inline]
    pub fn shift_left(&mut self, amount: usize) {
        self.mem_ptr = self.mem_ptr.saturating_sub(amount);
    }

    #[inline]
    pub fn shift_right(&mut self, amount: usize) -> Result<()> {
        self.mem_ptr += amount;

        if self.mem_ptr >= self.mem.len() {
            let mem_count = self.mem.len();
            let overflowed_count = self.mem_ptr - mem_count;
            bail!("memory overflowed: {mem_count} items => {overflowed_count} items")
        } else {
            Ok(())
        }
    }

    #[inline]
    pub fn shift_right_alt(&mut self, amount: usize) {
        self.mem_ptr += amount;
    }

    #[inline]
    pub fn jump_zero(&mut self, to: usize) {
        if self.get_cell() == 0 {
            self.pc = to;
        }
    }

    #[inline]
    pub fn jump_not_zero(&mut self, to: usize) {
        if self.get_cell() != 0 {
            self.pc = to;
        }
    }

    #[inline]
    pub fn print_chars(&mut self, amount: usize, stdout: &mut StdoutLock<'_>) {
        let ch = self.get_cell();
        for _ in 0..amount {
            stdout.write(&[ch]);
        }
    }

    #[inline]
    pub fn input_char(&mut self, _: usize) -> Result<()> {
        // self.input_chars ignores repetives.
        let ch = self.get_cell_mut();

        stdin().read_exact(array::from_mut(ch))?;

        Ok(())
    }

    /// Attribute inline(never) yield better performance?!
    /// Test with examples/mandelbrot.bf on i5-8300H faster for more than 2 seconds!
    #[inline(never)]
    pub fn run(&mut self) -> Result<()> {
        use OpCodeType::*;
        let stdout = stdout();
        let mut stdout = stdout.lock();

        while self.pc < self.program.len() {
            let (inst, data) = self.program[self.pc].to_tuple();

            match inst {
                Add => self.add_to_cell(data),
                Sub => self.sub_to_cell(data),
                ShiftLeft => self.shift_left(data),
                ShiftRight => self.shift_right(data)?,
                JmpZero => self.jump_zero(data),
                JmpNotZero => self.jump_not_zero(data),
                PrintChar => self.print_chars(data, &mut stdout),
                InputChar => self.input_char(data)?,
                _ => bail!("unimplemented instruction: {inst:?}"),
            }

            self.pc += 1;
        }

        Ok(())
    }
}
