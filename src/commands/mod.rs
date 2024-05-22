pub mod file;
pub mod directory;

use std::os::unix::fs::PermissionsExt;

use crate::config::Config;
use crate::commands::file::FileCommand;
use crate::commands::directory::DirectoryCommand;
use crate::error::Result;
use crate::error::Error;

pub trait Command {
    fn name(&self) -> &str;
    fn summary(&self) -> String;
    fn usage(&self) -> Result<String>;
    fn subcommands(&self) -> Vec<Box<dyn Command + '_>>;
    fn completions(&self) -> Result<i32>;
    fn invoke(&self) -> Result<i32>;
    fn help(&self) -> Result<String>;
}

pub fn subcommand(config: &Config, mut cliargs: Vec<String>) -> Result<Box<dyn Command + '_>> {
    let mut path = config.libexec_path();
    let mut names = Vec::new();

    if cliargs.is_empty() {
        if path.is_dir() {
            return Ok(Box::new(DirectoryCommand::top_level(names, path, config)?));
        }

        panic!("libexec is a file, not a directory");
    }

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
                let mut name_parts = vec![config.name.to_owned()];
                name_parts.append(&mut names.clone());
                return Ok(Box::new(DirectoryCommand::new(&name_parts.join(" "), names, path, config)?));
            }

            if path.metadata().unwrap().permissions().mode() & 0o111 == 0 {
                return Err(Error::NonExecutable(head.to_owned()));
            }

            return Ok(Box::new(FileCommand::new(names, path, cliargs, config)));
        }

        if path.is_dir() {
            continue;
        }

        if path.metadata().unwrap().permissions().mode() & 0o111 == 0 {
            return Err(Error::NonExecutable(head.to_owned()));
        }

        return Ok(Box::new(FileCommand::new(names, path, cliargs, config)));
    }
}

