use std::result;

use chumsky::prelude::Simple;

pub type Result<T> = result::Result<T, Error>;

#[derive(Clone)]
pub enum Error {
    NoCompletions,
    NonExecutable(String),
    SubCommandInterrupted,
    UnknownSubCommand(String),
    InvalidUsageString(Vec<Simple<char>>),
    InvalidUTF8,
}
