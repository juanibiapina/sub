extern crate regex;

use regex::Regex;
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

fn usage_parser() -> impl Parser<char, Usage, Error = Simple<char>> {
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

#[derive(PartialEq)]
enum Mode {
    Out,
    Usage,
    Help,
}

pub struct Docs {
    pub summary: String,
    pub usage: String,
    pub help: String,
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

pub fn extract_docs(path: &Path) -> Docs {
    lazy_static! {
        static ref SUMMARY_RE: Regex = Regex::new(r"^# Summary: (.*)$").unwrap();
        static ref USAGE_RE: Regex = Regex::new(r"^# (Usage: .*)$").unwrap();
        static ref INDENTED_RE: Regex = Regex::new(r"^# ( .*)$").unwrap();
        static ref EXTENDED_RE: Regex = Regex::new(r"^# (.*)$").unwrap();
    }

    let comment_block = extract_initial_comment_block(path);

    let mut summary = Vec::new();
    let mut usage = Vec::new();
    let mut help = Vec::new();

    let mut mode = Mode::Out;

    for line in comment_block.lines() {
        if mode == Mode::Out {
            if line == "#" {
                continue;
            }

            if let Some(caps) = SUMMARY_RE.captures(&line) {
                if let Some(m) = caps.get(1) {
                    summary.push(m.as_str().to_owned());
                    continue;
                }
            }

            if let Some(caps) = USAGE_RE.captures(&line) {
                if let Some(m) = caps.get(1) {
                    usage.push(m.as_str().to_owned());
                    mode = Mode::Usage;
                    continue;
                }
            }

            if let Some(caps) = EXTENDED_RE.captures(&line) {
                if let Some(m) = caps.get(1) {
                    help.push(m.as_str().to_owned());
                    mode = Mode::Help;
                    continue;
                }
            }
        }

        if mode == Mode::Usage {
            if line == "#" {
                usage.push("".to_owned());
                continue;
            }

            if let Some(caps) = INDENTED_RE.captures(&line) {
                if let Some(m) = caps.get(1) {
                    usage.push(m.as_str().to_owned());
                    continue;
                }
            }

            if let Some(caps) = EXTENDED_RE.captures(&line) {
                if let Some(m) = caps.get(1) {
                    help.push(m.as_str().to_owned());
                    mode = Mode::Help;
                    continue;
                }
            }
        }

        if mode == Mode::Help {
            if line == "#" {
                help.push("".to_owned());
                continue;
            }

            if let Some(caps) = EXTENDED_RE.captures(&line) {
                if let Some(m) = caps.get(1) {
                    help.push(m.as_str().to_owned());
                    continue;
                }
            }
        }
    }

    Docs {
        summary: summary.join("\n"),
        usage: usage.join("\n").trim().to_owned(),
        help: help.join("\n").trim().to_owned(),
    }
}

pub fn provides_completions(path: &Path) -> bool {
    let file = File::open(path).unwrap();

    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        if line == "# Provide completions" {
            return true;
        }
    }

    false
}
