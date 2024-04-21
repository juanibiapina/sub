use std::path::PathBuf;
use std::process;

use crate::error::{Error, Result};
use crate::parser;
use crate::engine::Engine;

pub trait Command {
    fn name(&self) -> &str;
    fn summary(&self) -> String;
    fn usage(&self) -> String;
    fn help(&self) -> String;
    fn completions(&self) -> Result<i32>;
    fn invoke(&self) -> Result<i32>;
}

pub fn internal_help(engine: &Engine, args: Vec<String>) -> InternalCommand {
    InternalCommand {
        name: "help",
        summary: "Display help for a sub command",
        help: "A command is considered documented if it starts with a comment block
            that has a `Summary:' or `Usage:' section. Usage instructions can
            span multiple lines as long as subsequent lines are indented.
            The remainder of the comment block is displayed as extended
            documentation.",
            args,
            engine,
            func: |engine: &Engine, args: Vec<String>| -> Result<i32> {
                let subcommand = engine.subcommand(args.clone())?;

                let usage = subcommand.usage();
                if !usage.is_empty() {
                    println!("{}", usage);
                    println!();
                }

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
    }
}

pub fn internal_commands(engine: &Engine, args: Vec<String>) -> InternalCommand {
    InternalCommand {
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
    }
}

pub fn internal_completions(engine: &Engine, args: Vec<String>) -> InternalCommand {
    InternalCommand {
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
    }
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
        let mut cmd = vec![self.engine.name().to_owned()];
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

    fn completions(&self) -> Result<i32> {
        if self.path.is_dir() {
            let commands = internal_commands(self.engine, self.names.clone());
            commands.invoke()
        } else {
            if parser::provides_completions(&self.path) {
                let mut command = process::Command::new(&self.path);

                command.arg("--complete");
                command.env(format!("_{}_ROOT", self.engine.name().to_uppercase()), self.engine.root());

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

            command.env(format!("_{}_ROOT", self.engine.name().to_uppercase()), self.engine.root());
            command.env(format!("_{}_CACHE", self.engine.name().to_uppercase()), self.engine.cache_directory());

            let status = command.status().unwrap();

            match status.code() {
                Some(code) => Ok(code),
                None => Err(Error::SubCommandInterrupted),
            }
        }
    }
}

impl<'e> Command for TopLevelCommand<'e> {
    fn name(&self) -> &str {
        &self.name
    }

    fn summary(&self) -> String {
        let mut readme_path = self.path.clone();
        readme_path.push("README");

        if readme_path.exists() {
            parser::extract_docs(&readme_path).0
        } else {
            "".to_owned()
        }
    }

    fn usage(&self) -> String {
        format!("Usage: {} [<subcommands>] [<args>]", self.name)
    }

    fn help(&self) -> String {
        let mut readme_path = self.path.clone();
        readme_path.push("README");

        if readme_path.exists() {
            parser::extract_docs(&readme_path).2
        } else {
            "".to_owned()
        }
    }

    fn completions(&self) -> Result<i32> {
        let commands = internal_commands(self.engine, Vec::new());
        commands.invoke()
    }

    fn invoke(&self) -> Result<i32> {
        let help_command = internal_help(self.engine, Vec::new());
        help_command.invoke()
    }
}

impl<'e> Command for InternalCommand<'e> {
    fn name(&self) -> &str {
        &self.name
    }

    fn summary(&self) -> String {
        self.summary.to_owned()
    }

    fn usage(&self) -> String {
        "".to_owned()
    }

    fn help(&self) -> String {
        self.help.to_owned()
    }

    fn completions(&self) -> Result<i32> {
        Ok(0) // TODO
    }

    fn invoke(&self) -> Result<i32> {
        (self.func)(self.engine, self.args.clone())
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
