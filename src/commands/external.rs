use std::fs;
use std::path::PathBuf;
use std::process;

use crate::parser;
use crate::error::{Error, Result};
use crate::engine::Engine;
use crate::commands::Command;
use crate::commands::internal::commands::internal_commands;
use crate::commands::internal::help::internal_help;

pub struct ExternalCommand<'e> {
    pub names: Vec<String>,
    pub path: PathBuf,
    pub args: Vec<String>,
    pub engine: &'e Engine,
}

impl<'e> Command for ExternalCommand<'e> {
    fn name(&self) -> &str {
        self.names.last().unwrap()
    }

    fn summary(&self) -> String {
        if self.path.is_dir() {
            let mut readme_path = self.path.clone();
            readme_path.push("README");

            if readme_path.exists() {
                parser::extract_docs(&readme_path).0
            } else {
                "".to_owned()
            }
        } else {
            parser::extract_docs(&self.path).0
        }
    }

    fn usage(&self) -> String {
        let mut cmd = vec![self.engine.config.name.to_owned()];
        cmd.extend(self.names.iter().map(|s| s.to_owned()));

        let cmd = cmd.join(" ");

        if self.path.is_dir() {
            vec!["Usage:", &cmd, "[<subcommands>]", "[<args>]"].join(" ")
        } else {
            let usage = parser::extract_docs(&self.path).1;
            if usage.is_empty() {
                format!("Usage: {}", cmd)
            } else {
                usage.replace("{cmd}", &cmd)
            }
        }
    }

    fn help(&self) -> String {
        if self.path.is_dir() {
            let mut readme_path = self.path.clone();
            readme_path.push("README");

            if readme_path.exists() {
                parser::extract_docs(&readme_path).2
            } else {
                "".to_owned()
            }
        } else {
            parser::extract_docs(&self.path).2
        }
    }

    fn subcommands(&self) -> Vec<Box<dyn Command + '_>> {
        let mut libexec_path = self.engine.config.libexec_path();
        libexec_path.extend(&self.names);

        let mut subcommands = Vec::new();

        if libexec_path.is_dir() {
            for entry in fs::read_dir(libexec_path).unwrap() {
                let name = entry.unwrap().file_name().to_str().unwrap().to_owned();

                let mut names = self.names.clone();
                names.push(name);

                if let Ok(subcommand) = self.engine.external_subcommand(names) {
                    subcommands.push(subcommand);
                }
            }
        }

        subcommands.sort_by(|c1, c2| c1.name().cmp(c2.name()));

        return subcommands;
    }

    fn completions(&self) -> Result<i32> {
        if self.path.is_dir() {
            let commands = internal_commands(self.engine, self.names.clone());
            commands.invoke()
        } else {
            if parser::provides_completions(&self.path) {
                let mut command = process::Command::new(&self.path);

                command.arg("--complete");
                command.env(format!("_{}_ROOT", self.engine.config.name.to_uppercase()), &self.engine.config.root);

                let status = command.status().unwrap();

                return match status.code() {
                    Some(code) => Ok(code),
                    None => Err(Error::SubCommandInterrupted),
                };
            }
            Ok(0)
        }
    }

    fn invoke(&self) -> Result<i32> {
        if !self.path.exists() {
            return Err(Error::UnknownSubCommand(self.names.last().unwrap().to_owned()));
        }

        if self.path.is_dir() {
            let help_command = internal_help(self.engine, self.names.clone());
            help_command.invoke()
        } else {
            let mut command = process::Command::new(&self.path);

            command.args(&self.args);

            command.env(format!("_{}_ROOT", self.engine.config.name.to_uppercase()), &self.engine.config.root);
            command.env(format!("_{}_CACHE", self.engine.config.name.to_uppercase()), &self.engine.config.cache_directory);

            let status = command.status().unwrap();

            match status.code() {
                Some(code) => Ok(code),
                None => Err(Error::SubCommandInterrupted),
            }
        }
    }
}
