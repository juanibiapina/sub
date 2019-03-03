extern crate regex;

use regex::Regex;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn extract_summary(path: &Path) -> String {
    let file = File::open(path).unwrap();
    lazy_static! {
        static ref SUMMARY_RE: Regex = Regex::new("^# Summary: (.*)$").unwrap();
    }
    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        if let Some(caps) = SUMMARY_RE.captures(&line) {
            if let Some(m) = caps.get(1) {
                return m.as_str().to_owned();
            }
        }
    }

    "".to_owned()
}

pub fn extract_usage(path: &Path) -> String {
    let file = File::open(path).unwrap();
    lazy_static! {
        static ref USAGE_RE: Regex = Regex::new("^# Usage: (.*)$").unwrap();
    }
    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        if let Some(caps) = USAGE_RE.captures(&line) {
            if let Some(m) = caps.get(1) {
                return m.as_str().to_owned();
            }
        }
    }

    "".to_owned()
}

pub fn extract_help(path: &Path) -> String {
    let file = File::open(path).unwrap();
    lazy_static! {
        static ref HELP_RE: Regex = Regex::new("^# Help: (.*)$").unwrap();
    }
    lazy_static! {
        static ref COMMENT_RE: Regex = Regex::new("^# (.*)$").unwrap();
    }
    let mut help_started = false;
    let mut help = String::new();

    for line in BufReader::new(file).lines() {
        let line = line.unwrap();

        if help_started {
            if line.starts_with('#') {
                if let Some(caps) = COMMENT_RE.captures(&line) {
                    if let Some(m) = caps.get(1) {
                        help.push('\n');
                        help.push_str(m.as_str());
                    } else {
                        break;
                    }
                } else {
                    help.push('\n');
                }
            } else {
                break;
            }
        } else {
            if let Some(caps) = HELP_RE.captures(&line) {
                if let Some(m) = caps.get(1) {
                    help_started = true;
                    help.push_str(m.as_str());
                }
            }
        }
    }

    help
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
