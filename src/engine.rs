use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::commands::Command;
use crate::commands::external::ExternalCommand;
use crate::commands::toplevel::TopLevelCommand;
use crate::commands::internal::help::internal_help;
use crate::commands::internal::commands::internal_commands;
use crate::commands::internal::completions::internal_completions;
use crate::error::Result;
use crate::error::Error;

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

    pub fn subcommand(&self, mut names: Vec<String>) -> Result<Box<dyn Command + '_>> {
        if names.is_empty() {
            return Ok(Box::new(TopLevelCommand {
                name: self.config.name.to_owned(),
                path: self.libexec_path(),
                engine: &self,
            }));
        }

        let name = &names[0];

        match name.as_ref() {
            "help" => Ok(Box::new(internal_help(&self, names.split_off(1)))),
            "commands" => Ok(Box::new(internal_commands(&self, names.split_off(1)))),
            "completions" => Ok(Box::new(internal_completions(&self, names.split_off(1)))),
            _ => {
                self.external_subcommand(names)
            },
        }
    }

    pub fn external_subcommand(&self, mut args: Vec<String>) -> Result<Box<dyn Command + '_>> {
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
                    return Ok(Box::new(ExternalCommand {
                        names,
                        path,
                        args,
                        engine: &self,
                    }));
                }

                if path.metadata().unwrap().permissions().mode() & 0o111 == 0 {
                    return Err(Error::NonExecutable(head.to_owned()));
                }

                return Ok(Box::new(ExternalCommand {
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

            return Ok(Box::new(ExternalCommand {
                names,
                path,
                args,
                engine: &self,
            }));
        }
    }

    pub fn libexec_path(&self) -> PathBuf {
        let mut path = self.config.root.clone();
        path.push("libexec");
        path
    }
}
