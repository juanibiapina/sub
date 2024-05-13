extern crate regex;

use regex::Regex;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

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
    Description,
}

pub struct Docs {
    pub summary: String,
    pub usage: String,
    pub description: String,
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
    let mut description = Vec::new();

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
                    description.push(m.as_str().to_owned());
                    mode = Mode::Description;
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

    Docs {
        summary: summary.join("\n"),
        usage: usage.join("\n").trim().to_owned(),
        description: description.join("\n").trim().to_owned(),
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
