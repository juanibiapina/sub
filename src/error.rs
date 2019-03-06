use std::result;

pub type Result<T> = result::Result<T, Error>;

pub enum Error {
    NoCompletions,
    NoSubCommand,
    NonExecutable(String),
    SubCommandInterrupted,
    UnknownSubCommand(String),
}
