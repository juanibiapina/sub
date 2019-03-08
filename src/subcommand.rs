use std::path::PathBuf;
use std::process::Command;

use crate::error::{Error, Result};
use crate::parser;
use crate::engine::Engine;

pub enum SubCommand<'e> {
    TopLevelCommand(TopLevelCommand<'e>),
    InternalCommand(InternalCommand<'e>),
    ExternalCommand(ExternalCommand<'e>),
}

impl<'e> SubCommand<'e> {
    pub fn internal_help(engine: &'e Engine, args: Vec<String>) -> SubCommand {
        SubCommand::InternalCommand(InternalCommand{
            name: "help",
            summary: "Display help for a sub command",
            help: "A command is considered documented if it starts with a comment block
that has a 'Summary:', or 'Help:' section. The help
section can span multiple lines as long as subsequent lines
are indented.", // TODO add Args: section
            args,
            engine,
            func: |engine: &Engine, args: Vec<String>| -> Result<i32> {
                let subcommand = engine.subcommand(args.clone())?;

                // TODO display usage information before help

                let summary = subcommand.summary();
                if !summary.is_empty() {
                    println!("{}", summary);
                    println!();
                }

                let help = subcommand.help();
                if !help.is_empty() {
                    println!("{}", help);
                }

                let subcommands = engine.subcommands(args.clone());
                if !subcommands.is_empty() {
                    println!();
                    println!("Available subcommands:");

                    let max_width = subcommands
                        .iter()
                        .map(|subcommand| subcommand.name())
                        .map(|name: &str| name.len())
                        .max()
                        .unwrap();

                    let width = max_width + 4;

                    for subcommand in subcommands {
                        println!("    {:width$}{}", subcommand.name(), subcommand.summary(), width = width);
                    }

                    println!();
                    let mut cs = args.clone();
                    cs.push("<command>".to_owned());
                    println!("Use '{} help {}' for information on a specific command.", engine.name(), cs.join(" "));
                }

                Ok(0)
            },
        })
    }

    pub fn internal_commands(engine: &'e Engine, args: Vec<String>) -> SubCommand {
        SubCommand::InternalCommand(InternalCommand{
            name: "commands",
            summary: "List available commands",
            help: "",
            args,
            engine,
            func: |engine: &Engine, args: Vec<String>| -> Result<i32> {
                for subcommand in engine.subcommands(args) {
                    println!("{}", subcommand.name());
                }

                Ok(0)
            },
        })
    }

    pub fn internal_completions(engine: &'e Engine, args: Vec<String>) -> SubCommand {
        SubCommand::InternalCommand(InternalCommand{
            name: "completions",
            summary: "List completions for a sub command",
            help: "",
            args,
            engine,
            func: |engine: &Engine, args: Vec<String>| -> Result<i32> {
                if let Ok(subcommand) = engine.subcommand(args) {
                    subcommand.completions()
                } else {
                    Ok(1)
                }
            },
        })
    }

    pub fn name(&self) -> &str {
        match self {
            SubCommand::TopLevelCommand(c) => &c.name,
            SubCommand::InternalCommand(c) => &c.name,
            SubCommand::ExternalCommand(c) => c.names.last().unwrap(),
        }
    }

    pub fn summary(&self) -> String {
        match self {
            SubCommand::TopLevelCommand(_) => "".to_owned(),
            SubCommand::InternalCommand(c) => c.summary.to_owned(),
            SubCommand::ExternalCommand(c) => {
                if c.path.is_dir() {
                    let mut readme_path = c.path.clone();
                    readme_path.push("README");

                    if readme_path.exists() {
                        parser::extract_summary(&readme_path)
                    } else {
                        "".to_owned()
                    }
                } else {
                    parser::extract_summary(&c.path)
                }
            },
        }
    }

    pub fn help(&self) -> String {
        match self {
            SubCommand::TopLevelCommand(c) => {
                format!("Usage: {} <command> [args]", c.name)
            },
            SubCommand::InternalCommand(c) => {
                c.help.to_owned()
            },
            SubCommand::ExternalCommand(c) => {
                if c.path.is_dir() {
                    let mut readme_path = c.path.clone();
                    readme_path.push("README");

                    if readme_path.exists() {
                        parser::extract_help(&readme_path)
                    } else {
                        "".to_owned()
                    }
                } else {
                    parser::extract_help(&c.path)
                }
            },
        }
    }

    pub fn completions(&self) -> Result<i32> {
        match self {
            SubCommand::TopLevelCommand(c) => {
                let commands = SubCommand::internal_commands(c.engine, Vec::new());
                commands.invoke()
            },
            SubCommand::InternalCommand(_) => Ok(0), // TODO
            SubCommand::ExternalCommand(c) => {
                if c.path.is_dir() {
                    let commands = SubCommand::internal_commands(c.engine, c.names.clone());
                    commands.invoke()
                } else {
                    if parser::provides_completions(&c.path) {
                        let mut command = Command::new(&c.path);

                        command.arg("--complete");
                        command.env(format!("_{}_ROOT", c.engine.name().to_uppercase()), c.engine.root());

                        let status = command.status().unwrap();

                        return match status.code() {
                            Some(code) => Ok(code),
                            None => Err(Error::SubCommandInterrupted),
                        };
                    }

                    Ok(0)


                }

            },
        }
    }

    pub fn invoke(&self) -> Result<i32> {
        match self {
            SubCommand::TopLevelCommand(c) => {
                let help_command = SubCommand::internal_help(c.engine, Vec::new());
                help_command.invoke()
            },
            SubCommand::InternalCommand(c) => (c.func)(c.engine, c.args.clone()),
            SubCommand::ExternalCommand(c) => {
                if !c.path.exists() {
                    return Err(Error::UnknownSubCommand(c.names.last().unwrap().to_owned()));
                }

                if c.path.is_dir() {
                    let help_command = SubCommand::internal_help(c.engine, c.names.clone());
                    help_command.invoke()
                } else {
                    let mut command = Command::new(&c.path);

                    command.args(&c.args);

                    command.env(format!("_{}_ROOT", c.engine.name().to_uppercase()), c.engine.root());

                    let status = command.status().unwrap();

                    match status.code() {
                        Some(code) => Ok(code),
                        None => Err(Error::SubCommandInterrupted),
                    }
                }
            },
        }
    }
}

pub struct TopLevelCommand<'e> {
    pub name: String,
    pub path: PathBuf,
    pub engine: &'e Engine,
}

pub struct InternalCommand<'e> {
    name: &'static str,
    summary: &'static str,
    help: &'static str,
    args: Vec<String>,
    engine: &'e Engine,
    func: fn(&Engine, Vec<String>) -> Result<i32>,
}

pub struct ExternalCommand<'e> {
    pub names: Vec<String>,
    pub path: PathBuf,
    pub args: Vec<String>,
    pub engine: &'e Engine,
}
