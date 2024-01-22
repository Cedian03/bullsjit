pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Generic(String),

    NoPathProvided,
    UnbalancedBrackets,

    IO(std::io::Error),
}