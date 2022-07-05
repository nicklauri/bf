use std::fs;

use crate::vm::{Vm, DEFAULT_VM_MEM_SIZE};
use clap::Parser;

mod lexer;
mod opcodes;
mod parser;
mod vm;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Args {
    file: String,

    #[clap(default_value_t = DEFAULT_VM_MEM_SIZE)]
    tape_size: usize,
}

fn run_file(path: &str) -> anyhow::Result<()> {
    let content = fs::read_to_string(&path)?;

    let mut vm = Vm::new(&content)?;

    Ok(vm.run()?)
}

fn main() {
    let args = Args::parse();

    let result = run_file(&args.file);

    if let Err(err) = result {
        eprintln!("error: {}", err);
    }
}
