extern crate regex;
use chumsky::prelude::*;

use regex::Regex;

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
    let cmd_token = just("{cmd}")
        .map(|_| Usage::CmdToken)
        .padded();

    cmd_token.then_ignore(end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cmd_token() {
        let input = "{cmd}";
        let result = usage_parser().parse(input).unwrap();
        assert_eq!(result, Usage::CmdToken);
    }
}

pub fn extract_usage(path: &Path) -> Result<Option<Usage>> {
    lazy_static! {
        static ref USAGELINE_RE: Regex = Regex::new(r"^# Usage: (.*)$").unwrap();
    }

    let comment_block = extract_initial_comment_block(path);

    for line in comment_block.lines() {
        if let Some(caps) = USAGELINE_RE.captures(&line) {
            if let Some(m) = caps.get(1) {
                match usage_parser().parse(m.as_str()) {
                    Ok(e) => return Ok(Some(e)),
                    Err(_) => return Err(Error::InvalidUsageString),
                }
            } else {
                return Err(Error::InvalidUsageString);
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
