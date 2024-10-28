pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Generic(String),

    UnbalancedBrackets,

    IO(std::io::Error),
}