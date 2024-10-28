mod common;
mod error;
mod methods;

use std::{fs, path};

use clap::{Parser, ValueEnum};

use common::*;
use methods::{compile_and_run, interpret};

use error::{Error, Result};

#[derive(Debug, clap::Parser)]
struct Args {
    mode: Mode,
    source: path::PathBuf,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Mode {
    Compile,
    Interpret,
}

fn parse(source: &str) -> Result<Vec<Instruction>> {
    let chars = source.as_bytes();

    let mut program = Vec::new();
    let mut stack = Vec::new();

    let mut i = 0;
    while let Some(ch) = chars.get(i) {
        match ch {
            b'>' | b'<' | b'+' | b'-' => {
                let j = i;
                while i < chars.len() && chars[i] == *ch {
                    i += 1
                }

                let n = i - j;
                match ch {
                    b'>' => program.push(Instruction::Right(n)),
                    b'<' => program.push(Instruction::Left(n)),
                    b'+' => program.push(Instruction::Increment(n as u8)),
                    b'-' => program.push(Instruction::Decrement(n as u8)),
                    _ => unreachable!(),
                }
            }
            b'.' => {
                program.push(Instruction::Output);
                i += 1;
            }
            b',' => {
                program.push(Instruction::Input);
                i += 1;
            }
            b'[' => {
                let index = program.len();
                stack.push(index);
                program.push(Instruction::JumpIfNonZero(index + 1));
                i += 1;
            }
            b']' => {
                let jump_if_non_zero = stack.pop().ok_or(Error::UnbalancedBrackets)?;
                let jump_if_zero = program.len();

                program.push(Instruction::JumpIfZero(jump_if_zero + 1));
                match program.get(jump_if_non_zero) {
                    Some(Instruction::JumpIfNonZero(_)) => {
                        program.swap(jump_if_non_zero, jump_if_zero)
                    }
                    _ => panic!("Invalid index on bracket stack"),
                }
                i += 1;
            }
            _ => i += 1,
        }
    }

    if stack.is_empty() {
        Ok(program)
    } else {
        Err(Error::UnbalancedBrackets)
    }
}

fn main() -> Result<()> {
    let args = dbg!(Args::parse());

    let source = fs::read_to_string(args.source).map_err(|err| {
        eprintln!("Provided file path doesn't exist");
        Error::IO(err)
    })?;

    let program = parse(&source).map_err(|err| {
        eprintln!("Invalid source code");
        err
    })?;

    match args.mode {
        Mode::Compile => compile_and_run(&program).map_err(|err| {
            eprintln!("JIT compiler ran into an error");
            err
        }),
        Mode::Interpret => interpret(&program).map_err(|err| {
            eprintln!("Interpreter ran into an error");
            err
        }),
    }
}