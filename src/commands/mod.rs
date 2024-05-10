pub mod internal;
pub mod file;
pub mod directory;
pub mod toplevel;

use std::os::unix::fs::PermissionsExt;

use crate::config::Config;
use crate::commands::file::FileCommand;
use crate::commands::directory::DirectoryCommand;
use crate::commands::toplevel::TopLevelCommand;
use crate::commands::internal::help::internal_help;
use crate::commands::internal::commands::internal_commands;
use crate::commands::internal::completions::internal_completions;
use crate::error::Result;
use crate::error::Error;

pub trait Command {
    fn name(&self) -> &str;
    fn summary(&self) -> String;
    fn usage(&self) -> String;
    fn help(&self) -> String;
    fn subcommands(&self) -> Vec<Box<dyn Command + '_>>;
    fn completions(&self) -> Result<i32>;
    fn invoke(&self) -> Result<i32>;
}

pub fn subcommand(config: &Config, mut names: Vec<String>) -> Result<Box<dyn Command + '_>> {
    if names.is_empty() {
        return Ok(Box::new(TopLevelCommand {
            name: config.name.to_owned(),
            path: config.libexec_path(),
            config,
        }));
    }

    let name = &names[0];

    match name.as_ref() {
        "help" => Ok(Box::new(internal_help(config, names.split_off(1)))),
        "commands" => Ok(Box::new(internal_commands(config, names.split_off(1)))),
        "completions" => Ok(Box::new(internal_completions(config, names.split_off(1)))),
        _ => {
            external_subcommand(config, names)
        },
    }
}

pub fn external_subcommand(config: &Config, mut args: Vec<String>) -> Result<Box<dyn Command + '_>> {
    let mut path = config.libexec_path();
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
                return Ok(Box::new(DirectoryCommand {
                    names,
                    path,
                    args,
                    config,
                }));
            }

            if path.metadata().unwrap().permissions().mode() & 0o111 == 0 {
                return Err(Error::NonExecutable(head.to_owned()));
            }

            return Ok(Box::new(FileCommand::new(names, path, args, config)?));
        }

        if path.is_dir() {
            continue;
        }

        if path.metadata().unwrap().permissions().mode() & 0o111 == 0 {
            return Err(Error::NonExecutable(head.to_owned()));
        }

        return Ok(Box::new(FileCommand::new(names, path, args, config)?));
    }
}

