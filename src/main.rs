mod error;

use std::{fs, env, io::{self, Read}};

use error::{Error, Result};

const SIZE: usize = 30000;

#[derive(Clone, Copy, Debug)]
enum Instruction {
    Right(usize),         // >
    Left(usize),          // <
    Increment(u8),        // +
    Decrement(u8),        // -
    Output,               // .
    Input,                // ,
    JumpIfZero(usize),    // [
    JumpIfNonZero(usize), // ]
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
                    _ => unreachable!()
                }
            },
            b'.' => {
                program.push(Instruction::Output);
                i += 1;
            },
            b',' => {
                program.push(Instruction::Input);
                i += 1;
            },
            b'[' => {
                let index = program.len();
                stack.push(index);
                program.push(Instruction::JumpIfNonZero(index + 1));
                i += 1;
            },
            b']' => {
                let jump_if_non_zero = stack.pop().ok_or(Error::UnbalancedBrackets)?;
                let jump_if_zero = program.len();

                program.push(Instruction::JumpIfZero(jump_if_zero + 1));
                match program.get(jump_if_non_zero) {
                    Some(Instruction::JumpIfNonZero(_)) => program.swap(jump_if_non_zero, jump_if_zero),
                    _ => panic!("Invalid index on bracket stack")
                }
                i += 1;
            },
            _ => i += 1
        }
    }

    if stack.is_empty() {
        Ok(program)
    } else {
        Err(Error::UnbalancedBrackets)
    }
}

fn interpret(program: &[Instruction]) -> Result<()> {
    let mut stdin = io::stdin().bytes();

    let mut memory = [0u8; SIZE];
    let mut cursor = 0;

    let mut ip = 0;
    while let Some(instruction) = program.get(ip) {
        match instruction {
            Instruction::Right(n) => {
                cursor += n;
                if cursor >= SIZE {
                    return Err(Error::CursorOverflow);
                }
                ip += 1;
            },
            Instruction::Left(n) => {
                if cursor < *n {
                    return Err(Error::CursorUnderflow);
                }
                cursor -= n;
                ip += 1;
            },
            Instruction::Increment(n) => {
                memory[cursor] = memory[cursor].wrapping_add(*n);
                ip += 1;
            },
            Instruction::Decrement(n) => {
                memory[cursor] = memory[cursor].wrapping_sub(*n);
                ip += 1;
            },
            Instruction::Output => {
                print!("{}", memory[cursor] as char);
                ip += 1;
            },
            Instruction::Input => {
                memory[cursor] = stdin.next().unwrap().map_err(|err| Error::IO(err))?;
                ip += 1;
            },
            Instruction::JumpIfZero(destination) => {
                if memory[cursor] == 0 {
                    ip = *destination;
                } else {
                    ip += 1;
                }
            },
            Instruction::JumpIfNonZero(destination) => {
                if memory[cursor] != 0 {
                    ip = *destination;
                } else {
                    ip += 1;
                }
            },
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);

    let path = args.next().ok_or_else(||  {
        println!("Usage: bullsjit <source.bf>");
        Error::NoPathProvided
    })?;

    let source = fs::read_to_string(path).map_err(|err| {
        println!("Provided file path doesn't exist");
        Error::IO(err)
    })?;

    let program = parse(&source).map_err(|err| {
        println!("Invalid source code");
        err
    })?;

    interpret(&program).map_err(|err| {
        println!("Runtime error");
        err
    })?;

    Ok(())
}