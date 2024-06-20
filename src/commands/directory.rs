use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;

use clap::Arg;

use crate::commands::subcommand;
use crate::commands::Command;
use crate::config::Config;
use crate::error::{Error, Result};
use crate::parser;
use crate::usage::Usage;

pub struct DirectoryCommand<'a> {
    names: Vec<String>,
    path: PathBuf,
    usage: Usage,
    config: &'a Config,
}

impl<'a> DirectoryCommand<'a> {
    pub fn top_level(names: Vec<String>, path: PathBuf, config: &'a Config) -> Self {
        let readme_path = path.join("README");

        let mut command = config.user_cli_command(&config.name);

        if readme_path.exists() {
            let docs = parser::extract_docs(&readme_path);

            if let Some(summary) = docs.summary {
                command = command.about(summary);
            }

            if let Some(description) = docs.description {
                command = command.after_help(description);
            }
        }

        let usage = Usage::new(command, HashMap::new(), None);

        return Self {
            names,
            path,
            usage,
            config,
        };
    }

    pub fn new(name: &str, names: Vec<String>, path: PathBuf, config: &'a Config) -> Self {
        let readme_path = path.join("README");

        let mut command = config.base_command(name);
        command = command.arg(Arg::new("commands_with_args").trailing_var_arg(true).allow_hyphen_values(true).num_args(..));

        if readme_path.exists() {
            let docs = parser::extract_docs(&readme_path);

            if let Some(summary) = docs.summary {
                command = command.about(summary);
            }

            if let Some(description) = docs.description {
                command = command.after_help(description);
            }
        }

        let usage = Usage::new(command, HashMap::new(), None);

        return Self {
            names,
            path,
            usage,
            config,
        };
    }
}

impl<'a> Command for DirectoryCommand<'a> {
    fn name(&self) -> &str {
        self.names.last().unwrap()
    }

    fn summary(&self) -> String {
        self.usage.summary()
    }

    fn usage(&self) -> Result<String> {
        Ok(self.usage.generate().to_string())
    }

    fn help(&self) -> Result<String> {
        let mut help = self.usage.help()?;

        let subcommands = self.subcommands();
        if !subcommands.is_empty() {
            help.push_str("\nAvailable subcommands:\n");

            let max_width = subcommands
                .iter()
                .map(|subcommand| subcommand.name())
                .map(|name| name.len())
                .max()
                .unwrap();

            let width = max_width + 4;

            for subcommand in subcommands {
                help.push_str(&format!(
                    "    {:width$}{}\n",
                    subcommand.name(),
                    subcommand.summary(),
                    width = width
                ));
            }
        }

        Ok(help)
    }

    fn subcommands(&self) -> Vec<Box<dyn Command + '_>> {
        let mut libexec_path = self.config.libexec_path();
        libexec_path.extend(&self.names);

        let mut subcommands = Vec::new();

        if libexec_path.is_dir() {
            for entry in fs::read_dir(libexec_path).unwrap() {
                let name = entry.unwrap().file_name().to_str().unwrap().to_owned();

                let mut names = self.names.clone();
                names.push(name);

                if let Ok(subcommand) = subcommand(self.config, names) {
                    subcommands.push(subcommand);
                }
            }
        }

        subcommands.sort_by(|c1, c2| c1.name().cmp(c2.name()));

        return subcommands;
    }

    fn completions(&self) -> Result<i32> {
        for command in self.subcommands() {
            println!("{}", command.name());
        }

        Ok(0)
    }

    fn invoke(&self) -> Result<i32> {
        if !self.path.exists() {
            return Err(Error::UnknownSubCommand(
                self.names.last().unwrap().to_owned(),
            ));
        }

        println!("{}", self.help()?);

        Ok(0)
    }

    fn validate(&self) -> Vec<(PathBuf, Error)> {
        let mut errors = Vec::new();

        for subcommand in self.subcommands() {
            errors.extend(subcommand.validate());
        }

        errors
    }
}
