pub const NO_CELLS: usize = 30000;

#[derive(Clone, Copy, Debug)]
pub enum Instruction {
    Right(usize),         // >
    Left(usize),          // <
    Increment(u8),        // +
    Decrement(u8),        // -
    Output,               // .
    Input,                // ,
    JumpIfZero(usize),    // [
    JumpIfNonZero(usize), // ]
}