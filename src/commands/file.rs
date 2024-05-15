use std::path::PathBuf;
use std::process;

use crate::config::Config;
use crate::usage::{self, Usage};
use crate::parser;
use crate::error::{Error, Result};
use crate::commands::Command;

pub struct FileCommand<'a> {
    names: Vec<String>,
    path: PathBuf,
    usage: Usage,
    args: Vec<String>,
    config: &'a Config,
}

impl<'a> FileCommand<'a> {
    pub fn new(names: Vec<String>, path: PathBuf, args: Vec<String>, config: &'a Config) -> Result<Self> {
        let usage = usage::extract_usage(&path)?.unwrap_or(Usage::default());

        return Ok(Self {
            names,
            path,
            usage,
            args,
            config,
        });
    }
}

impl<'a> Command for FileCommand<'a> {
    fn name(&self) -> &str {
        self.names.last().unwrap()
    }

    fn summary(&self) -> String {
        parser::extract_docs(&self.path).summary
    }

    fn usage(&self) -> String {
        let mut cmd = vec![self.config.name.to_owned()];
        cmd.extend(self.names.iter().map(|s| s.to_owned()));

        let cmd = cmd.join(" ");

        self.usage.generate(&cmd)
    }

    fn description(&self) -> String {
        parser::extract_docs(&self.path).description
    }

    fn subcommands(&self) -> Vec<Box<dyn Command + '_>> {
        let subcommands = Vec::new();
        return subcommands;
    }

    fn completions(&self) -> Result<i32> {
        if parser::provides_completions(&self.path) {
            let mut command = process::Command::new(&self.path);

            command.arg("--complete");
            command.env(format!("_{}_ROOT", self.config.name.to_uppercase()), &self.config.root);

            let status = command.status().unwrap();

            return match status.code() {
                Some(code) => Ok(code),
                None => Err(Error::SubCommandInterrupted),
            };
        }
        Ok(0)
    }

    fn invoke(&self) -> Result<i32> {
        if !self.path.exists() {
            return Err(Error::UnknownSubCommand(self.names.last().unwrap().to_owned()));
        }

        let mut command = process::Command::new(&self.path);

        command.args(&self.args);

        command.env(format!("_{}_ROOT", self.config.name.to_uppercase()), &self.config.root);
        command.env(format!("_{}_CACHE", self.config.name.to_uppercase()), &self.config.cache_directory);

        let status = command.status().unwrap();

        match status.code() {
            Some(code) => Ok(code),
            None => Err(Error::SubCommandInterrupted),
        }
    }
}
