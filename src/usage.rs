extern crate regex;

use chumsky::prelude::*;
use clap::{Command, Arg};

use std::path::Path;

use crate::parser;
use crate::error::{Error, Result};
use crate::config::Config;

#[derive(Debug, PartialEq)]
pub enum Argument {
}

#[derive(Debug, PartialEq)]
struct UsageLang {
    arguments: Vec<Argument>,
}

fn usage_parser() -> impl Parser<char, Vec::<Argument>, Error = Simple<char>> {
    let prefix = just("# Usage:").padded();

    let cmd_token = just("{cmd}").padded().map(|_| Vec::<Argument>::new());

    prefix.ignore_then(cmd_token).then_ignore(end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cmd_token() {
        let input = "# Usage: {cmd}";
        let result = usage_parser().parse(input).unwrap();
        assert_eq!(result, Vec::<Argument>::new());
    }
}

pub struct Usage {
    command: Command,
}

impl Usage {
    pub fn from_command(command: Command) -> Self {
        Self {
            command,
        }
    }

    fn new(config: &Config, usage_lang: UsageLang, cmd: &str) -> Self {
        Self {
            command: config.base_command(cmd),
        }
    }

    pub fn generate(&self) -> String {
        self.command.clone().render_usage().ansi().to_string()
    }

    pub fn command(&self) -> Command {
        self.command.clone()
    }
}

pub fn extract_usage(config: &Config, path: &Path, cmd: &str) -> Result<Usage> {
    let docs = parser::extract_docs(&path);

    let mut command = config.base_command(cmd);

    if let Some(summary) = docs.summary {
        command = command.about(summary);
    }

    if let Some(description) = docs.description {
        command = command.after_help(description);
    }

    if let Some(line) = docs.usage {
        match usage_parser().parse(line) {
            Ok(arguments) => {
                // TODO add arguments to command
            },
            Err(e) => return Err(Error::InvalidUsageString(e)),
        }
    } else {
        command = command.arg(Arg::new("args").trailing_var_arg(true).num_args(..).allow_hyphen_values(true));
    }

    return Ok(Usage::from_command(command));
}
