use crate::error::Result;
use crate::config::Config;
use crate::commands::Command;

pub mod help;
pub mod commands;
pub mod completions;

pub struct InternalCommand<'a> {
    pub name: &'static str,
    pub summary: &'static str,
    pub description: &'static str,
    pub args: Vec<String>,
    pub config: &'a Config,
    pub func: fn(&Config, Vec<String>) -> Result<i32>,
}

impl<'a> Command for InternalCommand<'a> {
    fn name(&self) -> &str {
        &self.name
    }

    fn summary(&self) -> String {
        self.summary.to_owned()
    }

    fn usage(&self) -> String {
        "".to_owned() // TODO
    }

    fn description(&self) -> String {
        self.description.to_owned()
    }

    fn subcommands(&self) -> Vec<Box<dyn Command>> {
        // none of the internal subcommands currently have any subcommands
        return Vec::new();
    }

    fn completions(&self) -> Result<i32> {
        Ok(0) // TODO
    }

    fn invoke(&self) -> Result<i32> {
        (self.func)(self.config, self.args.clone())
    }
}
