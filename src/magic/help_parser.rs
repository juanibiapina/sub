extern crate regex;

use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use super::error::{MagicError, Result};
use super::metadata::Docs;

pub fn extract_initial_comment_block(path: &Path) -> Result<String> {
    let file = File::open(path).map_err(|e| MagicError::IoError(e.to_string()))?;

    let mut lines = Vec::new();

    for line in BufReader::new(file).lines() {
        let line = line.map_err(|e| MagicError::IoError(e.to_string()))?;

        if line.starts_with("#") {
            lines.push(line);
        } else {
            break;
        }
    }

    Ok(lines.join("\n"))
}

#[derive(PartialEq)]
enum Mode {
    Out,
    Options,
    Description,
}

pub fn extract_docs(path: &Path) -> Result<Docs> {
    lazy_static! {
        static ref SUMMARY_RE: Regex = Regex::new(r"^# Summary: (.*)$").unwrap();
        static ref INDENTED_RE: Regex = Regex::new(r"^# ( .*)$").unwrap();
        static ref EXTENDED_RE: Regex = Regex::new(r"^# (.*)$").unwrap();
    }

    let comment_block = extract_initial_comment_block(path)?;

    let mut summary = None;
    let mut usage = None;
    let mut options = Vec::new();
    let mut description = Vec::new();

    let mut mode = Mode::Out;

    for line in comment_block.lines() {
        if mode == Mode::Out {
            if line == "#" {
                continue;
            }

            if let Some(caps) = SUMMARY_RE.captures(&line) {
                if let Some(m) = caps.get(1) {
                    summary = Some(m.as_str().trim().to_owned());
                    continue;
                }
            }

            if line.starts_with("# Usage:") {
                usage = Some(line.to_owned());
                continue;
            }

            if line == "# Options:" {
                mode = Mode::Options;
                continue;
            }

            if let Some(caps) = EXTENDED_RE.captures(&line) {
                if let Some(m) = caps.get(1) {
                    description.push(m.as_str().trim().to_owned());
                    mode = Mode::Description;
                    continue;
                }
            }
        }

        if mode == Mode::Options {
            if line == "#" {
                mode = Mode::Out;
            }

            if let Some(caps) = INDENTED_RE.captures(&line) {
                if let Some(m) = caps.get(1) {
                    options.push(m.as_str().trim().to_owned());
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
                    description.push(m.as_str().trim().to_owned());
                    continue;
                }
            }
        }
    }

    Ok(Docs {
        summary,
        usage,
        options,
        description: if description.is_empty() { None } else { Some(description.join("\n")) },
    })
}

pub fn provides_completions(path: &Path) -> Result<bool> {
    let file = File::open(path).map_err(|e| MagicError::IoError(e.to_string()))?;

    for line in BufReader::new(file).lines() {
        let line = line.map_err(|e| MagicError::IoError(e.to_string()))?;
        if line == "# Provide completions" {
            return Ok(true);
        }
    }

    Ok(false)
}