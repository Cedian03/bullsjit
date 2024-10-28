use std::{io::{self, Read}, num::Wrapping};

use crate::common::*;
use crate::error::{Error, Result};

pub fn interpret(program: &[Instruction]) -> Result<()> {
    let mut stdin = io::stdin().bytes();

    let mut cells = [Wrapping(0u8); NO_CELLS];
    let mut cursor = 0;

    let mut ip = 0;
    while let Some(instruction) = program.get(ip) {
        match instruction {
            Instruction::Right(n) => {
                cursor += n;
                if cursor >= NO_CELLS {
                    return Err(Error::Generic("Cursor overflow".to_string()));
                }
                ip += 1;
            }
            Instruction::Left(n) => {
                if cursor < *n {
                    return Err(Error::Generic("Cursor underflow".to_string()));
                }
                cursor -= n;
                ip += 1;
            }
            Instruction::Increment(n) => {
                cells[cursor] += n;
                ip += 1;
            }
            Instruction::Decrement(n) => {
                cells[cursor] -= n;
                ip += 1;
            }
            Instruction::Output => {
                print!("{}", char::from(cells[cursor].0));
                ip += 1;
            }
            Instruction::Input => {
                cells[cursor] = Wrapping(stdin.next().unwrap().map_err(|err| Error::IO(err))?);
                ip += 1;
            }
            Instruction::JumpIfZero(destination) => {
                if cells[cursor].0 == 0 {
                    ip = *destination;
                } else {
                    ip += 1;
                }
            }
            Instruction::JumpIfNonZero(destination) => {
                if cells[cursor].0 != 0 {
                    ip = *destination;
                } else {
                    ip += 1;
                }
            }
        }
    }

    Ok(())
}
