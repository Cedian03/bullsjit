use std::{alloc, ptr};

use crate::common::{Instruction, NO_CELLS};
use crate::error::{Error, Result};

const ALIGNMENT: usize = 4096;

pub fn compile_and_run(program: &[Instruction]) -> Result<()> {
    let func = compile(program)?;

    let mut cells = [0; NO_CELLS];

    unsafe { func(cells.as_mut_ptr()); };

    // TODO: Drop function pointer

    Ok(())
}

fn compile(program: &[Instruction]) -> Result<unsafe fn(*mut u8)> {
    let mut bin = Vec::new();
    let mut stack = Vec::new();

    for instruction in program {
        match instruction {
            Instruction::Right(n) => {
                bin.extend_from_slice(&[0x48, 0x81, 0xC7]);               // add rdi,
                bin.extend_from_slice(&(*n as u32).to_le_bytes());        // n
            }
            Instruction::Left(n) => {
                bin.extend_from_slice(&[0x48, 0x81, 0xEF]);               // sub rdi,
                bin.extend_from_slice(&(*n as u32).to_le_bytes());        // n
            }
            Instruction::Increment(n) => {
                bin.extend_from_slice(&[0x80, 0x07, *n])                  // add byte[rdi], n
            }
            Instruction::Decrement(n) => {
                bin.extend_from_slice(&[0x80, 0x2F, *n])                  // sub byte[rdi], n
            }
            Instruction::Output => {
                bin.extend_from_slice(&[
                    0x48, 0x89, 0xFE,                         // mov rsi, rdi
                    0x48, 0xC7, 0xC0, 0x01, 0x00, 0x00, 0x00, // mov rax, 1 (sys_read)
                    0x48, 0xC7, 0xC7, 0x01, 0x00, 0x00, 0x00, // mov rdi, 1 (stdout)
                    0x48, 0xC7, 0xC2, 0x01, 0x00, 0x00, 0x00, // mov rdx, 1
                    0x0F, 0x05,                               // syscall
                    0x48, 0x89, 0xF7                          // mov rdi, rsi
                ]);
            }
            Instruction::Input => {
                bin.extend_from_slice(&[
                    0x48, 0x89, 0xFE,                         // mov rsi, rdi
                    0x48, 0xC7, 0xC0, 0x00, 0x00, 0x00, 0x00, // mov rax, 0 (sys_write)
                    0x48, 0xC7, 0xC7, 0x00, 0x00, 0x00, 0x00, // mov rdi, 0 (stdin)
                    0x48, 0xC7, 0xC2, 0x01, 0x00, 0x00, 0x00, // mov rdx, 1
                    0x0F, 0x05,                               // syscall
                    0x48, 0x89, 0xF7                          // mov rdi, rsi
                ]);
            }
            Instruction::JumpIfZero(_) => {
                bin.extend_from_slice(&[
                    0x80, 0x3F, 0x00,                         // cmp byte[rdi], 0
                    0x0F, 0x84,                               // je
                    0x00, 0x00, 0x00, 0x00                    // PLACEHOLDER
                ]);

                stack.push(bin.len());
            }
            Instruction::JumpIfNonZero(_) => {
                bin.extend_from_slice(&[
                    0x80, 0x3F, 0x00,                         // cmp byte[rdi], 0
                    0x0F, 0x85,                               // jne
                    0x00, 0x00, 0x00, 0x00                    // PLACEHOLDER
                ]);

                let idx_jz = stack.pop().ok_or(Error::UnbalancedBrackets)?;
                let idx_jnz = bin.len();

                let relative = idx_jnz as i32 -  idx_jz as i32;

                bin.splice((idx_jz - 4)..idx_jz, relative.to_le_bytes());
                bin.splice((idx_jnz - 4)..idx_jnz, (-relative).to_le_bytes());
            }
        }
    }
    bin.push(0xC3); // ret

    if stack.is_empty() {
        let size = bin.len();

        let func_layout = std::alloc::Layout::from_size_align(size, ALIGNMENT).unwrap();

        let func = unsafe {
            let func_ptr = alloc::alloc(func_layout);
            
            if func_ptr.is_null() {
                return Err(Error::Generic(String::from("Failed to allocate memory for function")));
            }

            ptr::copy_nonoverlapping(bin.as_ptr(), func_ptr, size);

            if libc::mprotect(func_ptr as *mut libc::c_void, size, libc::PROT_READ | libc::PROT_EXEC) != 0 {
                return Err(Error::Generic(String::from("Failed to set memory protection")))
            }

            std::mem::transmute(func_ptr) 
        };

        Ok(func)
    } else {
        Err(Error::UnbalancedBrackets)
    }
}
