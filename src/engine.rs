use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::subcommand::SubCommand;
use crate::error::Result;
use crate::error::Error;
use crate::parser;

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
                if let Some(subcommand)  = SubCommand::from_entry(&entry.unwrap()) {
                    subcommands.push(subcommand);
                }
            }
        }

        subcommands.push(SubCommand::internal_help());
        subcommands.push(SubCommand::internal_commands());

        subcommands.sort_by(|c1, c2| c1.name().cmp(c2.name()));

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

        let usage = parser::extract_usage(&command_path);
        if !usage.is_empty() {
            println!("Usage: {}\n", usage);
        }

        let help = parser::extract_help(&command_path);
        if help.is_empty() {
            let summary = parser::extract_summary(&command_path);
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

        if parser::provides_completions(&command_path) {
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
            println!("{}", subcommand.name());
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
                .map(|subcommand| subcommand.name())
                .map(|name: &str| name.len())
                .max()
                .unwrap();

            let width = max_width + 4;

            for subcommand in subcommands {
                println!("    {:width$}{}", subcommand.name(), subcommand.summary(), width = width);
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
