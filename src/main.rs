#![allow(warnings)]
use lexer::Lexer;
use std::{env, fs, time::Instant};

use crate::{parser::Parser, vm::Vm};

mod codegen;
mod lexer;
mod opcodes;
mod parser;
mod vm;

fn usage() {
    let program_name = env::args().nth(0).unwrap();
    println!("Usage: {} <file>", program_name);
}

fn run_file(path: &str) -> anyhow::Result<()> {
    let content = fs::read_to_string(&path)?;

    let inst = Instant::now();
    let mut vm = Vm::new(&content)?;
    println!("creating VM tooks: {:?}", inst.elapsed());
    println!("program instructions count: {}", vm.program().len());

    let inst = Instant::now();
    vm.run()?;
    println!("run program tooks: {:?}", inst.elapsed());

    Ok(())
}

fn handle_error(err: anyhow::Result<()>) {
    if let Err(err) = err {
        eprintln!("error: {}", err);
    }
}

fn main() {
    match env::args().nth(1) {
        Some(path) => {
            handle_error(run_file(&path));
        }
        None => usage(),
    }
}
