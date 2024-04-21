use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::subcommand::{SubCommand, ExternalCommand, TopLevelCommand, internal_completions, internal_help, internal_commands};
use crate::error::Result;
use crate::error::Error;

pub struct Config {
    pub name: String,
    pub root: PathBuf,
    pub cache_directory: PathBuf,
}

pub struct Engine {
    config: Config,
}

impl Engine {
    pub fn new(config: Config) -> Engine {
        Engine {
            config,
        }
    }

    pub fn name(&self) -> &str {
        &self.config.name
    }

    pub fn root(&self) -> &Path {
        &self.config.root
    }

    pub fn cache_directory(&self) -> &Path {
        &self.config.cache_directory
    }

    pub fn subcommand(&self, mut args: Vec<String>) -> Result<SubCommand> {
        if args.is_empty() {
            return Ok(SubCommand::TopLevelCommand(TopLevelCommand{
                name: self.config.name.to_owned(),
                path: self.libexec_path(),
                engine: &self,
            }));
        }

        let name = &args[0];

        match name.as_ref() {
            "help" => Ok(internal_help(&self, args.split_off(1))),
            "commands" => Ok(internal_commands(&self, args.split_off(1))),
            "completions" => Ok(internal_completions(&self, args.split_off(1))),
            _ => {
                self.external_subcommand(args)
            },
        }
    }

    fn external_subcommand(&self, mut args: Vec<String>) -> Result<SubCommand> {
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

            if args.is_empty() {
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
            subcommands.push(internal_help(&self, Vec::new()));
            subcommands.push(internal_commands(&self, Vec::new()));
        }

        subcommands.sort_by(|c1, c2| c1.name().cmp(c2.name()));

        subcommands
    }

    pub fn display_unknown_subcommand(&self, name: &str) {
        println!("{}: no such sub command '{}'", self.config.name, name);
    }

    fn libexec_path(&self) -> PathBuf {
        let mut path = self.config.root.clone();
        path.push("libexec");
        path
    }
}
