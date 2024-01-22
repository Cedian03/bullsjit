pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NoPathProvided,
    UnbalancedBrackets,
    IO(std::io::Error),

    CursorOverflow,
    CursorUnderflow,
}