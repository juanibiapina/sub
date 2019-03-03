extern crate regex;

use regex::Regex;

use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::path::{PathBuf, Path};
use std::process::Command;

use crate::subcommand::SubCommand;
use crate::error::Result;
use crate::error::Error;

pub struct Engine {
    name: String,
    root: PathBuf,
    args: Vec<String>,
}

impl Engine {
    pub fn new(name: String, root: PathBuf, args: Vec<String>) -> Engine {
        Engine {
            name,
            root,
            args,
        }
    }

    pub fn run(&self) -> Result<i32> {
        if self.args.is_empty() {
            self.display_help();
            return Err(Error::NoSubCommand);
        }

        let mut args = self.args.clone();

        let command_args = {
            if args.len() > 1 {
                args.drain(1..).collect()
            } else {
                Vec::new()
            }
        };
        let command_name = args.pop().unwrap();

        if command_name == "help" {
            if command_args.is_empty() {
                self.display_help();
            } else {
                self.display_help_for_command(&command_args[0]);
            }
            return Ok(0);
        }

        if command_name == "commands" {
            self.display_commands();
            return Ok(0);
        }

        if command_name == "completions" {
            if command_args.len() != 1 {
                self.display_commands();
                return Ok(0)
            }
            return self.display_completions(&command_args[0]);
        }

        self.run_subcommand(&command_name, &command_args)
    }

    fn run_subcommand(&self, name: &str, args: &[String]) -> Result<i32> {
        let command_path = self.command_path(&name);

        if !command_path.exists() {
            self.display_unknown_subcommand(&name);
            return Err(Error::UnknownSubCommand);
        }

        let mut command = Command::new(command_path);

        command.args(args);

        command.env(format!("_{}_ROOT", self.name.to_uppercase()), &self.root);

        let status = command.status().unwrap();

        match status.code() {
            Some(code) => Ok(code),
            None => Err(Error::SubCommandInterrupted),
        }
    }

    fn collect_subcommands(&self) -> Vec<SubCommand> {
        let libexec_path = self.libexec_path();

        let mut subcommands = Vec::new();

        if libexec_path.is_dir() {
            for entry in fs::read_dir(libexec_path).unwrap() {
                let entry = entry.unwrap();
                let name = entry.file_name().into_string().unwrap();

                if name.starts_with('.') {
                    continue;
                }

                if entry.metadata().unwrap().permissions().mode() & 0o111 == 0 {
                    continue;
                }

                let summary = extract_summary(&entry.path());
                let usage = extract_usage(&entry.path());
                let help = extract_help(&entry.path());
                let subcommand = SubCommand::new(name, summary, usage, help);

                subcommands.push(subcommand);
            }
        }

        subcommands.push(SubCommand::new(
                "commands".to_owned(),
                "List available commands".to_owned(),
                format!("Usage: {} commands", self.name),
                "".to_owned(),
            )
        );

        subcommands.push(SubCommand::new(
                "help".to_owned(),
                "Display help for a sub command".to_owned(),
                format!("Usage: {} help <command>", self.name),
                "".to_owned(),
            )
        );

        subcommands.sort_by(|c1, c2| c1.name.cmp(&c2.name));

        subcommands
    }

    fn display_unknown_subcommand(&self, name: &str) {
        println!("{}: no such sub command '{}'", self.name, name);
    }

    fn display_help_for_command(&self, command_name: &str) {
        let command_path = self.command_path(command_name);

        if !command_path.exists() {
            self.display_unknown_subcommand(command_name);
            return
        }

        let usage = extract_usage(&command_path);
        if !usage.is_empty() {
            println!("Usage: {}\n", usage);
        }

        let help = extract_help(&command_path);
        if help.is_empty() {
            let summary = extract_summary(&command_path);
            if !summary.is_empty() {
                println!("{}", summary);
            }
        } else {
            println!("{}", help);
        }
    }

    fn display_completions(&self, command_name: &str) -> Result<i32> {
        let command_path = self.command_path(command_name);

        if !command_path.exists() {
            return Err(Error::UnknownSubCommand);
        }

        if provides_completions(&command_path) {
            let mut command = Command::new(command_path);

            command.arg("--complete");
            command.env(format!("_{}_ROOT", self.name.to_uppercase()), &self.root);

            let status = command.status().unwrap();

            return match status.code() {
                Some(code) => Ok(code),
                None => Err(Error::SubCommandInterrupted),
            };
        }

        Ok(0)
    }

    fn display_commands(&self) {
        for subcommand in self.collect_subcommands() {
            println!("{}", subcommand.name);
        }
    }

    fn display_help(&self) {
        println!("Usage: {} <command> [args]", self.name);
        println!();

        let subcommands = self.collect_subcommands();
        if !subcommands.is_empty() {
            println!("Available commands:");

            let max_width = subcommands
                .iter()
                .map(|subcommand| subcommand.name.as_ref())
                .map(|name: &str| name.len())
                .max()
                .unwrap();

            let width = max_width + 4;

            for subcommand in subcommands {
                println!("    {:width$}{}", subcommand.name, subcommand.summary, width = width);
            }

            println!();
        }

        println!("Use '{} help <command>' for information on a specific command.", self.name);
    }

    fn command_path(&self, command_name: &str) -> PathBuf {
        let mut libexec_path = self.libexec_path();
        libexec_path.push(command_name);
        libexec_path
    }

    fn libexec_path(&self) -> PathBuf {
        let mut path = self.root.clone();
        path.push("libexec");
        path
    }
}

fn extract_summary(path: &Path) -> String {
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

fn extract_usage(path: &Path) -> String {
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

fn extract_help(path: &Path) -> String {
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

fn provides_completions(path: &Path) -> bool {
    let file = File::open(path).unwrap();

    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        if line == "# Provide completions" {
            return true;
        }
    }

    false
}
