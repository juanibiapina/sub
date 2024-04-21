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

    fn completions(&self) -> Result<i32> {
        let commands = internal_commands(self.engine, Vec::new());
        commands.invoke()
    }

    fn invoke(&self) -> Result<i32> {
        let help_command = internal_help(self.engine, Vec::new());
        help_command.invoke()
    }
}
