extern crate regex;

use chumsky::prelude::*;
use clap::Command;

use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::error::{Error, Result};
use crate::config::Config;

#[derive(Debug, PartialEq)]
enum UsageLang {
    CmdToken,
}

fn usage_parser() -> impl Parser<char, UsageLang, Error = Simple<char>> {
    let prefix = just("# Usage:").padded();

    let cmd_token = just("{cmd}")
        .map(|_| UsageLang::CmdToken)
        .padded();

    prefix.ignore_then(cmd_token).then_ignore(end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cmd_token() {
        let input = "# Usage: {cmd}";
        let result = usage_parser().parse(input).unwrap();
        assert_eq!(result, UsageLang::CmdToken);
    }
}

pub struct Usage {
    command: Command,
}

impl Usage {
    fn new(config: &Config, usage_lang: UsageLang, cmd: &str) -> Self {
        Self {
            command: config.clap_command(cmd),
        }
    }

    fn default(config: &Config, cmd: &str) -> Self {
        Self {
            command: config.clap_command(cmd),
        }
    }

    pub fn generate(&self) -> impl Display {
        self.command.clone().render_usage().ansi().to_string()
    }
}

pub fn extract_usage(config: &Config, path: &Path, cmd: &str) -> Result<Usage> {
    let comment_block = extract_initial_comment_block(path);

    for line in comment_block.lines() {
        if line.starts_with("# Usage:") {
            match usage_parser().parse(line) {
                Ok(usage_lang) => return Ok(Usage::new(config, usage_lang, cmd)),
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
