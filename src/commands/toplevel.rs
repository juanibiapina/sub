use std::fs;
use std::path::PathBuf;

use crate::config::Config;
use crate::error::Result;
use crate::parser;
use crate::commands::Command;
use crate::commands::external_subcommand;

pub struct TopLevelCommand<'a> {
    pub name: String,
    pub path: PathBuf,
    pub config: &'a Config,
}

impl<'a> Command for TopLevelCommand<'a> {
    fn name(&self) -> &str {
        &self.name
    }

    fn summary(&self) -> String {
        let mut readme_path = self.path.clone();
        readme_path.push("README");

        if readme_path.exists() {
            parser::extract_docs(&readme_path).summary
        } else {
            "".to_owned()
        }
    }

    fn usage(&self) -> String {
        format!("Usage: {} [<subcommands>] [<args>]", self.name)
    }

    fn description(&self) -> String {
        let mut readme_path = self.path.clone();
        readme_path.push("README");

        if readme_path.exists() {
            parser::extract_docs(&readme_path).description
        } else {
            "".to_owned()
        }
    }

    fn subcommands(&self) -> Vec<Box<dyn Command + '_>> {
        let libexec_path = self.config.libexec_path();

        let mut subcommands = Vec::new();

        if libexec_path.is_dir() {
            for entry in fs::read_dir(libexec_path).unwrap() {
                let name = entry.unwrap().file_name().to_str().unwrap().to_owned();

                if let Ok(subcommand) = external_subcommand(self.config, vec![name]) {
                    subcommands.push(subcommand);
                }
            }
        }

        subcommands.sort_by(|c1, c2| c1.name().cmp(c2.name()));

        subcommands
    }

    fn completions(&self) -> Result<i32> {
        for command in self.subcommands() {
            println!("{}", command.name());
        }

        Ok(0)
    }

    fn invoke(&self) -> Result<i32> {
        println!("{}", self.help());

        Ok(0)
    }
}
