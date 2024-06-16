use std::result;
use std::io;

use chumsky::prelude::Simple;

pub type Result<T> = result::Result<T, Error>;

#[derive(Clone)]
pub enum Error {
    NoCompletions,
    NonExecutable(String),
    NoLibexecDir,
    SubCommandIoError(std::rc::Rc<io::Error>),
    SubCommandInterrupted,
    UnknownSubCommand(String),
    InvalidUsageString(Vec<Simple<char>>),
    InvalidUTF8,
}
