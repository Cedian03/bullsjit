mod error;

use std::{
    alloc, env, fs, io::{self, Read}, mem, ops::Neg, ptr
};

use error::{Error, Result};

const NO_CELLS: usize = 30000;
const PAGE_SIZE: usize = 4096;

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

fn interpret(program: &[Instruction]) -> Result<()> {
    let mut stdin = io::stdin().bytes();

    let mut cells = [0u8; NO_CELLS];
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
                cells[cursor] = cells[cursor].wrapping_add(*n);
                ip += 1;
            }
            Instruction::Decrement(n) => {
                cells[cursor] = cells[cursor].wrapping_sub(*n);
                ip += 1;
            }
            Instruction::Output => {
                print!("{}", cells[cursor] as char);
                ip += 1;
            }
            Instruction::Input => {
                cells[cursor] = stdin.next().unwrap().map_err(|err| Error::IO(err))?;
                ip += 1;
            }
            Instruction::JumpIfZero(destination) => {
                if cells[cursor] == 0 {
                    ip = *destination;
                } else {
                    ip += 1;
                }
            }
            Instruction::JumpIfNonZero(destination) => {
                if cells[cursor] != 0 {
                    ip = *destination;
                } else {
                    ip += 1;
                }
            }
        }
    }
    Ok(())
}

fn compile_and_run(program: &[Instruction]) -> Result<()> {
    let mut bin = Vec::new();
    let mut stack = Vec::new();

    for instruction in program {
        match instruction {
            Instruction::Right(n) => {
                bin.extend([0x48, 0x81, 0xC7]);               // add rdi,
                bin.extend((*n as u32).to_le_bytes());        // n
            }
            Instruction::Left(n) => {
                bin.extend([0x48, 0x81, 0xEF]);               // sub rdi,
                bin.extend((*n as u32).to_le_bytes());        // n
            }
            Instruction::Increment(n) => {
                bin.extend([0x80, 0x07, *n])                  // add byte[rdi], n
            }
            Instruction::Decrement(n) => {
                bin.extend([0x80, 0x2F, *n])                  // sub byte[rdi], n
            }
            Instruction::Output => {
                bin.extend([
                    0x48, 0x89, 0xFE,                         // mov rsi, rdi
                    0x48, 0xC7, 0xC0, 0x01, 0x00, 0x00, 0x00, // mov rax, 1 (sys_read)
                    0x48, 0xC7, 0xC7, 0x01, 0x00, 0x00, 0x00, // mov rdi, 1 (stdout)
                    0x48, 0xC7, 0xC2, 0x01, 0x00, 0x00, 0x00, // mov rdx, 1
                    0x0F, 0x05,                               // syscall
                    0x48, 0x89, 0xF7                          // mov rdi, rsi
                ]);
            }
            Instruction::Input => {
                bin.extend([
                    0x48, 0x89, 0xFE,                         // mov rsi, rdi
                    0x48, 0xC7, 0xC0, 0x00, 0x00, 0x00, 0x00, // mov rax, 0 (sys_write)
                    0x48, 0xC7, 0xC7, 0x00, 0x00, 0x00, 0x00, // mov rdi, 0 (stdin)
                    0x48, 0xC7, 0xC2, 0x01, 0x00, 0x00, 0x00, // mov rdx, 1
                    0x0F, 0x05,                               // syscall
                    0x48, 0x89, 0xF7                          // mov rdi, rsi
                ]);
            }
            Instruction::JumpIfZero(_) => {
                bin.extend([
                    0x80, 0x3F, 0x00,                         // cmp byte[rdi], 0
                    0x0F, 0x84,                               // je
                    0x00, 0x00, 0x00, 0x00                    // PLACEHOLDER
                ]);

                stack.push(bin.len());
            }
            Instruction::JumpIfNonZero(_) => {
                bin.extend([
                    0x80, 0x3F, 0x00,                         // cmp byte[rdi], 0
                    0x0F, 0x85,                               // jne
                    0x00, 0x00, 0x00, 0x00                    // PLACEHOLDER
                ]);

                let idx_jz = stack.pop().ok_or(Error::UnbalancedBrackets)?;
                let idx_jnz = bin.len();

                let relative = idx_jnz as i32 -  idx_jz as i32;

                bin.splice((idx_jz - 4)..idx_jz, relative.to_le_bytes());
                bin.splice((idx_jnz - 4)..idx_jnz, relative.neg().to_le_bytes());
            }
        }
    }
    bin.push(0xC3); // ret

    if !stack.is_empty() {
        return Err(Error::UnbalancedBrackets);
    }

    let func_size = bin.len();

    let func_layout = alloc::Layout::from_size_align(func_size, PAGE_SIZE).unwrap();
    let cell_layout = alloc::Layout::from_size_align(NO_CELLS, PAGE_SIZE).unwrap();

    let protection = libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC;

    let func_ptr;
    let cell_ptr;
    unsafe {
        func_ptr = alloc::alloc(func_layout);
        cell_ptr = alloc::alloc_zeroed(cell_layout);
    }

    if func_ptr.is_null() {
        return Err(Error::Generic("Failed to allocate memory for function".to_string()));
    }

    if cell_ptr.is_null() {
        return Err(Error::Generic("Failed to allocate memory for cells".to_string()));
    }

    let function: fn(*mut u8) -> ();
    unsafe {
        if libc::mprotect(func_ptr as *mut libc::c_void, func_size, protection) != 0 {
            alloc::dealloc(func_ptr, func_layout);
            alloc::dealloc(cell_ptr, cell_layout);
            return Err(Error::Generic("Failed to set memory protection".to_string()));
        }

        ptr::copy_nonoverlapping(bin.as_ptr(), func_ptr, func_size);
        function = mem::transmute(func_ptr);
    }

    function(cell_ptr);

    unsafe {
        alloc::dealloc(func_ptr, func_layout);
        alloc::dealloc(cell_ptr, cell_layout);
    }

    Ok(())
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);

    let path = args.next().ok_or_else(|| {
        println!("Usage: bullsjit [--interpret] <source.bf>");
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

    if true {
        compile_and_run(&program).map_err(|err| {
            println!("JIT compiler ran into an error");
            err
        })?;
    } else {
        interpret(&program).map_err(|err| {
            println!("Interpreter ran into an error");
            err
        })?;
    };

    Ok(())
}