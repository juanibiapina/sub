extern crate regex;

use regex::Regex;

use chumsky::prelude::*;
use clap::{Command, Arg};

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

#[derive(PartialEq)]
enum Mode {
    Out,
    Description,
}

pub fn extract_usage(config: &Config, path: &Path, cmd: &str) -> Result<Usage> {
    lazy_static! {
        static ref SUMMARY_RE: Regex = Regex::new(r"^# Summary: (.*)$").unwrap();
        static ref INDENTED_RE: Regex = Regex::new(r"^# ( .*)$").unwrap();
        static ref EXTENDED_RE: Regex = Regex::new(r"^# (.*)$").unwrap();
    }

    let comment_block = extract_initial_comment_block(path);

    let mut command = config.base_command(cmd);

    let mut mode = Mode::Out;
    let mut found_usage = false;
    let mut description = Vec::new();

    for line in comment_block.lines() {
        if mode == Mode::Out {
            if line == "#" {
                continue;
            }

            if let Some(caps) = SUMMARY_RE.captures(&line) {
                if let Some(m) = caps.get(1) {
                    command = command.about(m.as_str().to_owned());
                    continue;
                }
            }

            if line.starts_with("# Usage:") {
                match usage_parser().parse(line) {
                    Ok(arguments) => {
                        found_usage = true;
                        // TODO add arguments to command
                    },
                    Err(e) => return Err(Error::InvalidUsageString(e)),
                }

                continue;
            }

            if let Some(caps) = EXTENDED_RE.captures(&line) {
                if let Some(m) = caps.get(1) {
                    description.push(m.as_str().to_owned());
                    mode = Mode::Description;
                    continue;
                }
            }
        }

        if mode == Mode::Description {
            if line == "#" {
                description.push("".to_owned());
                continue;
            }

            if let Some(caps) = EXTENDED_RE.captures(&line) {
                if let Some(m) = caps.get(1) {
                    description.push(m.as_str().to_owned());
                    continue;
                }
            }
        }
    }

    command = command.long_about(description.join("\n"));

    if !found_usage {
        command = command.arg(Arg::new("args").trailing_var_arg(true).num_args(..).allow_hyphen_values(true));
    }

    return Ok(Usage::from_command(command));
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
