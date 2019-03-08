use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::subcommand::{SubCommand, ExternalCommand, TopLevelCommand};
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn run(&self) -> Result<i32> {
        self.subcommand(self.args.clone())?.invoke()
    }

    pub fn subcommand(&self, mut args: Vec<String>) -> Result<SubCommand> {
        if args.is_empty() {
            return Ok(SubCommand::TopLevelCommand(TopLevelCommand{
                name: self.name.to_owned(),
                path: self.libexec_path(),
                engine: &self,
            }));
        }

        let name = &args[0];

        match name.as_ref() {
            "help" => Ok(SubCommand::internal_help(&self, args.split_off(1))),
            "commands" => Ok(SubCommand::internal_commands(&self, args.split_off(1))),
            "completions" => Ok(SubCommand::internal_completions(&self, args.split_off(1))),
            _ => {
                self.external_subcommand(args)
            },
        }
    }

    fn external_subcommand(&self, mut args: Vec<String>) -> Result<SubCommand> {
        if args.is_empty() {
            return Ok(SubCommand::TopLevelCommand(TopLevelCommand{
                name: self.name.to_owned(),
                path: self.libexec_path(),
                engine: &self,
            }));
        }

        let mut path = self.libexec_path();
        let mut names = Vec::new();

        loop {
            let head = args[0].clone();

            if head.starts_with('.') {
                return Err(Error::UnknownSubCommand(head.to_owned()));
            }

            path.push(&head);

            if !path.exists() {
                return Err(Error::UnknownSubCommand(head));
            }

            names.push(head.to_owned());

            args = args.split_off(1);

            if args.len() == 0 {
                if path.is_dir() {
                    return Ok(SubCommand::ExternalCommand(ExternalCommand{
                        names,
                        path,
                        args,
                        engine: &self,
                    }));
                }

                if path.metadata().unwrap().permissions().mode() & 0o111 == 0 {
                    return Err(Error::NonExecutable(head.to_owned()));
                }

                return Ok(SubCommand::ExternalCommand(ExternalCommand{
                    names,
                    path,
                    args,
                    engine: &self,
                }));
            }

            if path.is_dir() {
                continue;
            }

            if path.metadata().unwrap().permissions().mode() & 0o111 == 0 {
                return Err(Error::NonExecutable(head.to_owned()));
            }

            return Ok(SubCommand::ExternalCommand(ExternalCommand{
                names,
                path,
                args,
                engine: &self,
            }));
        }
    }

    pub fn subcommands(&self, names: Vec<String>) -> Vec<SubCommand> {
        let include_internal = names.is_empty();

        let mut libexec_path = self.libexec_path();
        libexec_path.extend(&names);

        let mut subcommands = Vec::new();

        if libexec_path.is_dir() {
            for entry in fs::read_dir(libexec_path).unwrap() {
                let name = entry.unwrap().file_name().to_str().unwrap().to_owned();

                let mut names = names.clone();
                names.push(name);

                if let Ok(subcommand) = self.external_subcommand(names) {
                    subcommands.push(subcommand);
                }
            }
        }

        if include_internal {
            subcommands.push(SubCommand::internal_help(&self, Vec::new()));
            subcommands.push(SubCommand::internal_commands(&self, Vec::new()));
        }

        subcommands.sort_by(|c1, c2| c1.name().cmp(c2.name()));

        subcommands
    }

    pub fn display_unknown_subcommand(&self, name: &str) {
        println!("{}: no such sub command '{}'", self.name, name);
    }

    pub fn display_completions(&self, command_name: &str) -> Result<i32> {
        let command_path = self.command_path(command_name);

        if !command_path.exists() {
            return Err(Error::NoCompletions);
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

    pub fn display_help(&self) {
        println!("Usage: {} <command> [args]", self.name);
        println!();

        let subcommands = self.subcommands(Vec::new());
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
