mod error;

use std::{fs, env, io::{self, Read}};

use error::{Error, Result};

const SIZE: usize = 30000;

#[derive(Clone, Copy, Debug)]
enum Instruction {
    Right,                // >
    Left,                 // <
    Increment,            // +
    Decrement,            // -
    Output,               // .
    Input,                // ,
    JumpIfZero(usize),    // [
    JumpIfNonZero(usize), // ]
}

fn parse(source: &str) -> Result<Vec<Instruction>> {
    let mut program = Vec::new();
    let mut stack = Vec::new();

    for ch in source.chars() {
        match ch {
            '>' => program.push(Instruction::Right),
            '<' => program.push(Instruction::Left),
            '+' => program.push(Instruction::Increment),
            '-' => program.push(Instruction::Decrement),
            '.' => program.push(Instruction::Output),
            ',' => program.push(Instruction::Input),
            '[' => {
                let index = program.len();
                stack.push(index);
                program.push(Instruction::JumpIfNonZero(index + 1))
            }
            ']' => {
                let jump_if_non_zero = stack.pop().ok_or(Error::UnbalancedBrackets)?;
                let jump_if_zero = program.len();

                program.push(Instruction::JumpIfZero(jump_if_zero + 1));
                match program.get(jump_if_non_zero) {
                    Some(Instruction::JumpIfNonZero(_)) => program.swap(jump_if_non_zero, jump_if_zero),
                    _ => panic!("Invalid index on bracket stack")
                }
            },
            _ => {}
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
            Instruction::Right => {
                cursor += 1;
                if cursor >= SIZE {
                    cursor = 0;
                }
                ip += 1;
            },
            Instruction::Left => {
                if cursor == 0 {
                    cursor = SIZE;
                }
                cursor -= 1;
                ip += 1;
            },
            Instruction::Increment => {
                memory[cursor] = memory[cursor].wrapping_add(1);
                ip += 1;
            },
            Instruction::Decrement => {
                memory[cursor] = memory[cursor].wrapping_sub(1);
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