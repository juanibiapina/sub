pub mod file;
pub mod directory;
pub mod toplevel;

use std::os::unix::fs::PermissionsExt;

use crate::config::Config;
use crate::commands::file::FileCommand;
use crate::commands::directory::DirectoryCommand;
use crate::commands::toplevel::TopLevelCommand;
use crate::error::Result;
use crate::error::Error;

pub trait Command {
    fn name(&self) -> &str;
    fn summary(&self) -> String;
    fn usage(&self) -> String;
    fn description(&self) -> String;
    fn subcommands(&self) -> Vec<Box<dyn Command + '_>>;
    fn completions(&self) -> Result<i32>;
    fn invoke(&self) -> Result<i32>;

    fn help(&self) -> String {
        let mut help = String::new();

        let usage = self.usage();
        if !usage.is_empty() {
            help.push_str(&usage);
            help.push_str("\n\n");
        }

        let summary = self.summary();
        if !summary.is_empty() {
            help.push_str(&summary);
            help.push_str("\n\n");
        }

        let description = self.description();
        if !description.is_empty() {
            help.push_str(&description);
        }

        let subcommands = self.subcommands();
        if !subcommands.is_empty() {
            help.push_str("\n\nAvailable subcommands:\n");

            let max_width = subcommands
                .iter()
                .map(|subcommand| subcommand.name())
                .map(|name| name.len())
                .max()
                .unwrap();

            let width = max_width + 4;

            for subcommand in subcommands {
                help.push_str(&format!("    {:width$}{}\n", subcommand.name(), subcommand.summary(), width = width));
            }
        }

        help
    }
}

pub fn subcommand(config: &Config, cliargs: Vec<String>) -> Result<Box<dyn Command + '_>> {
    if cliargs.is_empty() {
        return Ok(Box::new(TopLevelCommand {
            name: config.name.to_owned(),
            path: config.libexec_path(),
            config,
        }));
    }

    external_subcommand(config, cliargs)
}

pub fn external_subcommand(config: &Config, mut cliargs: Vec<String>) -> Result<Box<dyn Command + '_>> {
    let mut path = config.libexec_path();
    let mut names = Vec::new();

    loop {
        let head = cliargs[0].clone();

        if head.starts_with('.') {
            return Err(Error::UnknownSubCommand(head.to_owned()));
        }

        path.push(&head);

        if !path.exists() {
            return Err(Error::UnknownSubCommand(head));
        }

        names.push(head.to_owned());

        cliargs = cliargs.split_off(1);

        if cliargs.is_empty() {
            if path.is_dir() {
                return Ok(Box::new(DirectoryCommand {
                    names,
                    path,
                    args: cliargs,
                    config,
                }));
            }

            if path.metadata().unwrap().permissions().mode() & 0o111 == 0 {
                return Err(Error::NonExecutable(head.to_owned()));
            }

            return Ok(Box::new(FileCommand::new(names, path, cliargs, config)?));
        }

        if path.is_dir() {
            continue;
        }

        if path.metadata().unwrap().permissions().mode() & 0o111 == 0 {
            return Err(Error::NonExecutable(head.to_owned()));
        }

        return Ok(Box::new(FileCommand::new(names, path, cliargs, config)?));
    }
}

