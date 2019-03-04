use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

use crate::error::{Error, Result};
use crate::parser;
use crate::engine::Engine;

pub enum SubCommand {
    InternalCommand(InternalCommand),
    ExternalCommand(ExternalCommand),
    NestedCommand(NestedCommand),
}

impl SubCommand {
    fn from_dir(path: PathBuf) -> Option<SubCommand> {
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();

        Some(SubCommand::NestedCommand(NestedCommand {
            name,
            path,
        }))
    }

    pub fn from_path(path: PathBuf) -> Option<SubCommand> {
        if !path.exists() {
            return None;
        }

        let name = path.file_name().unwrap().to_str().unwrap().to_string();

        if name.starts_with('.') {
            return None;
        }

        if path.is_dir() {
            return SubCommand::from_dir(path);
        }

        if path.metadata().unwrap().permissions().mode() & 0o111 == 0 {
            return None;
        }

        Some(SubCommand::ExternalCommand(ExternalCommand {
            name,
            path,
        }))
    }

    pub fn internal_help() -> SubCommand {
        SubCommand::InternalCommand(InternalCommand{
            name: "help",
            summary: "Display help for a sub command",
            help: "A command is considered documented if it starts with a comment block
that has a 'Summary:', or 'Help:' section. The help
section can span multiple lines as long as subsequent lines
are indented.", // TODO add Args: section
            func: |engine: &Engine, args: &[String]| -> Result<i32> {
                if args.is_empty() {
                    engine.display_help();
                } else {
                    engine.display_help_for_command(&args[0]);
                }
                return Ok(0);
            },
        })
    }

    pub fn internal_commands() -> SubCommand {
        SubCommand::InternalCommand(InternalCommand{
            name: "commands",
            summary: "List available commands",
            help: "",
            func: |engine: &Engine, _args: &[String]| -> Result<i32> {
                for subcommand in engine.subcommands() {
                    println!("{}", subcommand.name());
                }

                return Ok(0);
            },
        })
    }

    pub fn internal_completions() -> SubCommand {
        SubCommand::InternalCommand(InternalCommand{
            name: "completions",
            summary: "List completions for a sub command",
            help: "",
            func: |engine: &Engine, args: &[String]| -> Result<i32> {
                if args.len() != 1 {
                    SubCommand::internal_commands().invoke(engine, args)
                } else {
                    engine.display_completions(&args[0])
                }
            },
        })
    }

    pub fn name(&self) -> &str {
        match self {
            SubCommand::InternalCommand(c) => &c.name,
            SubCommand::ExternalCommand(c) => &c.name,
            SubCommand::NestedCommand(c) => &c.name,
        }
    }

    pub fn summary(&self) -> String {
        match self {
            SubCommand::InternalCommand(c) => c.summary.to_owned(),
            SubCommand::ExternalCommand(c) => parser::extract_summary(&c.path),
            SubCommand::NestedCommand(c) => {
                let mut readme_path = c.path.clone();
                readme_path.push("README");

                if readme_path.exists() {
                    parser::extract_summary(&readme_path)
                } else {
                    "".to_owned()
                }
            },
        }
    }

    pub fn help(&self) -> String {
        match self {
            SubCommand::InternalCommand(c) => {
                c.help.to_owned()
            },
            SubCommand::ExternalCommand(c) => {
                parser::extract_help(&c.path)
            },
            SubCommand::NestedCommand(c) => {
                let mut readme_path = c.path.clone();
                readme_path.push("README");

                if readme_path.exists() {
                    parser::extract_help(&readme_path)
                } else {
                    "".to_owned()
                }
            },
        }
    }

    pub fn invoke(&self, engine: &Engine, args: &[String]) -> Result<i32> {
        match self {
            SubCommand::InternalCommand(c) => (c.func)(engine, args),
            SubCommand::ExternalCommand(c) => {
                if !c.path.exists() {
                    engine.display_unknown_subcommand(&c.name);
                    return Err(Error::UnknownSubCommand);
                }

                let mut command = Command::new(&c.path);

                command.args(args);

                command.env(format!("_{}_ROOT", engine.name().to_uppercase()), engine.root());

                let status = command.status().unwrap();

                match status.code() {
                    Some(code) => Ok(code),
                    None => Err(Error::SubCommandInterrupted),
                }
            },
            SubCommand::NestedCommand(c) => {
                if args.is_empty() {
                    let help_command = SubCommand::internal_help();
                    help_command.invoke(engine, &[c.name.to_owned()])
                } else {
                    Ok(0)
                }
            },
        }
    }
}

pub struct InternalCommand {
    name: &'static str,
    summary: &'static str,
    help: &'static str,
    func: fn(&Engine, &[String]) -> Result<i32>,
}

pub struct ExternalCommand {
    name: String,
    path: PathBuf,
}

pub struct NestedCommand {
    name: String,
    path: PathBuf,
}
