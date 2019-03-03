extern crate clap;
extern crate itertools;
extern crate regex;

use clap::{App, AppSettings, Arg, SubCommand as ClapSubCommand};
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
        if self.args.len() == 0 {
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
            if command_args.len() == 0 {
                self.display_help();
            } else {
                self.display_help_for_command(&command_args[0]);
            }
            return Ok(0);
        }

        let command_path = self.command_path(&command_name);

        if !command_path.exists() {
            self.display_unknown_subcommand(&command_name);
            return Err(Error::UnknownSubCommand);
        }

        let mut command = Command::new(command_path);
        command.args(command_args);

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

                if name.starts_with(".") {
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

        subcommands
    }

    fn display_unknown_subcommand(&self, name: &str) {
        println!("{}: no such sub command '{}'", self.name, name);
    }

    fn display_help_for_command(&self, command_name: &str) {
        let command_path = self.command_path(command_name);

        if !command_path.exists() {
            println!("{}: no such sub command '{}'", self.name, command_name);
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

    fn display_help(&self) {
        let subcommands = self.collect_subcommands();

        let mut app = App::new(self.name.as_ref())
            .bin_name(self.name.as_ref())
            .setting(AppSettings::ColoredHelp)
            .setting(AppSettings::NoBinaryName)
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .setting(AppSettings::DisableVersion)
            .setting(AppSettings::VersionlessSubcommands);

        for subcommand in subcommands.iter() {
            app = app.subcommand(
                ClapSubCommand::with_name(subcommand.name.as_ref())
                .about(subcommand.summary.as_ref())
                .setting(AppSettings::TrailingVarArg)
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(Arg::with_name("args")
                     .hidden(true)
                     .multiple(true)),
                     );
        }

        app.print_help().unwrap();
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
            if line.starts_with("#") {
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
