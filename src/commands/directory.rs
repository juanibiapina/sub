use std::fs;
use std::path::PathBuf;

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
    pub fn new(name: &str, names: Vec<String>, path: PathBuf, config: &'a Config) -> Result<Self> {
        let readme_path = path.join("README");

        let mut command = config.user_cli_command(name);

        if readme_path.exists() {
            let docs = parser::extract_docs(&readme_path);

            if let Some(summary) = docs.summary {
                command = command.about(summary);
            }

            if let Some(description) = docs.description {
                command = command.long_about(description);
            }
        }

        let usage = Usage::from_command(command);

        return Ok(Self {
            names,
            path,
            usage,
            config,
        });
    }
}

impl<'a> Command for DirectoryCommand<'a> {
    fn name(&self) -> &str {
        self.names.last().unwrap()
    }

    fn summary(&self) -> String {
        self.usage.command().get_about().map(|s| s.ansi().to_string()).unwrap_or_default()
    }

    fn usage(&self) -> String {
        self.usage.generate().to_string()
    }

    fn description(&self) -> String {
        self.usage.command().get_long_about().map(|s| s.ansi().to_string()).unwrap_or_default()
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

        println!("{}", self.help());

        Ok(0)
    }
}
