extern crate regex;
use chumsky::prelude::*;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::error::{Error, Result};

#[derive(Debug, PartialEq)]
pub enum Usage {
    CmdToken,
}

impl Usage {
    pub fn generate(&self, cmd: &str) -> String {
        match self {
            Usage::CmdToken => format!("Usage: {}", cmd),
        }
    }
}

pub fn usage_parser() -> impl Parser<char, Usage, Error = Simple<char>> {
    let prefix = just("# Usage:").padded();

    let cmd_token = just("{cmd}")
        .map(|_| Usage::CmdToken)
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
        assert_eq!(result, Usage::CmdToken);
    }
}

pub fn extract_usage(path: &Path) -> Result<Option<Usage>> {
    let comment_block = extract_initial_comment_block(path);

    for line in comment_block.lines() {
        if line.starts_with("# Usage:") {
            match usage_parser().parse(line) {
                Ok(e) => return Ok(Some(e)),
                Err(e) => return Err(Error::InvalidUsageString(e)),
            }
        }
    }

    return Ok(None);
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
