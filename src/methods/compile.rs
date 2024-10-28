use std::{alloc, mem, ptr};

use crate::common::{Instruction, NO_CELLS};
use crate::error::{Error, Result};

const ALIGNMENT: usize = 4096;

pub fn compile_and_run(program: &[Instruction]) -> Result<()> {
    let func = compile(program)?;

    let mut cells = [0u8; NO_CELLS];
    unsafe { func(cells.as_mut_ptr()); }

    Ok(())
}

fn compile(program: &[Instruction]) -> Result<unsafe fn(*mut u8)> {
    let mut bin = AVec::new(ALIGNMENT);
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

                let relative = idx_jnz as i32 - idx_jz as i32;
                
                unsafe {
                    bin.splice(idx_jz - mem::size_of::<i32>(), relative.to_le_bytes());
                    bin.splice(idx_jnz - mem::size_of::<i32>(), (-relative).to_le_bytes());
                }
            }                
        }
    }
    bin.push(0xC3); // ret

    if stack.is_empty() {
        Ok(unsafe {
            let bin = bin.leak();
            let func_ptr = bin.as_mut_ptr();
    
            if libc::mprotect(func_ptr as *mut libc::c_void, bin.len(), libc::PROT_READ | libc::PROT_EXEC) != 0 {
                return Err(Error::Generic(String::from("Failed to set memory protection")))
            }

            mem::transmute(func_ptr) 
        })
    } else {
        Err(Error::UnbalancedBrackets)
    }
}

struct AVec {
    ptr: *mut u8,
    size: usize,
    len: usize,
    align: usize,
}

impl AVec {
    pub fn new(align: usize) -> AVec {
        AVec { ptr: ptr::null_mut(), size: 0, len: 0, align }
    }

    pub fn push(&mut self, item: u8) {
        if self.len >= self.size {
            unsafe { self.resize(self.size * 2); };
        }

        unsafe { ptr::write(self.ptr.add(self.len), item) };
        self.len += 1;
    }

    pub fn extend(&mut self, iter: impl IntoIterator<Item = u8>) {
        for item in iter.into_iter() {
            self.push(item);
        }
    }

    pub unsafe fn splice(&mut self, start: usize, iter: impl IntoIterator<Item = u8>) {
        for (i, item) in (start..).zip(iter.into_iter()) {
            unsafe { ptr::write(self.ptr.add(i), item); };
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub unsafe fn leak(self) -> &'static mut [u8] {
        let me = mem::ManuallyDrop::new(self);
        unsafe { &mut *ptr::slice_from_raw_parts_mut(me.ptr, me.len) }
    }

    unsafe fn resize(&mut self, new_size: usize) {
        let new_size = usize::max(new_size, 8);
        self.ptr = unsafe { alloc::realloc(self.ptr, self.layout(), new_size) };
        self.size = new_size;
    }

    fn layout(&self) -> alloc::Layout {
        alloc::Layout::from_size_align(self.size, self.align)
            .expect("Invalid memory layout")
    }
}

impl Drop for AVec {
    fn drop(&mut self) {
        unsafe { alloc::dealloc(self.ptr, self.layout()); };
        self.ptr = ptr::null_mut();
    }
}
