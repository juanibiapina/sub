extern crate regex;

use chumsky::prelude::*;
use clap::{Command, Arg};

use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

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
    cmd: String,
    command: Command,
}

impl Usage {
    fn new(config: &Config, usage_lang: UsageLang, cmd: &str) -> Self {
        Self {
            cmd: cmd.to_string(),
            command: config.clap_command(cmd),
        }
    }

    fn default(config: &Config, cmd: &str) -> Self {
        Self {
            cmd: cmd.to_string(),
            command: config.clap_command(cmd).arg(Arg::new("args").trailing_var_arg(true).num_args(..).allow_hyphen_values(true)),
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
    let comment_block = extract_initial_comment_block(path);

    for line in comment_block.lines() {
        if line.starts_with("# Usage:") {
            match usage_parser().parse(line) {
                Ok(arguments) => return Ok(Usage::new(config, UsageLang { arguments }, cmd)),
                Err(e) => return Err(Error::InvalidUsageString(e)),
            }
        }
    }

    return Ok(Usage::default(config, cmd));
}

fn extract_initial_comment_block(path: &Path) -> String {
    let file = File::open(path).unwrap();

    let mut lines = Vec::new();

    for line in BufReader::new(file).lines() {
        let line = line.unwrap();

        if line.starts_with("#") {
            lines.push(line);
        } else {
            break;
        }
    }

    lines.join("\n")
}
