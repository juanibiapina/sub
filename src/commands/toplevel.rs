use std::fs;
use std::path::PathBuf;

use crate::error::Result;
use crate::parser;
use crate::engine::Engine;
use crate::commands::Command;
use crate::commands::internal::help::internal_help;
use crate::commands::internal::commands::internal_commands;

pub struct TopLevelCommand<'e> {
    pub name: String,
    pub path: PathBuf,
    pub engine: &'e Engine,
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

    fn subcommands(&self) -> Vec<Box<dyn Command + '_>> {
        let libexec_path = self.engine.config.libexec_path();

        let mut subcommands = Vec::new();

        if libexec_path.is_dir() {
            for entry in fs::read_dir(libexec_path).unwrap() {
                let name = entry.unwrap().file_name().to_str().unwrap().to_owned();

                if let Ok(subcommand) = self.engine.external_subcommand(vec![name]) {
                    subcommands.push(subcommand);
                }
            }
        }

        subcommands.push(Box::new(internal_help(self.engine, Vec::new())));
        subcommands.push(Box::new(internal_commands(self.engine, Vec::new())));

        subcommands.sort_by(|c1, c2| c1.name().cmp(c2.name()));

        subcommands
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
