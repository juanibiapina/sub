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
                if args.is_empty() {
                    engine.display_help();
                } else {
                    let subcommand = engine.subcommand(args)?;

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
                }
                return Ok(0);
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

                return Ok(0);
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
                if args.len() != 1 {
                    SubCommand::internal_commands(engine, args).invoke()
                } else {
                    engine.display_completions(&args[0])
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
            SubCommand::TopLevelCommand(_) => "".to_owned(),
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
