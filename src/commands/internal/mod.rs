use crate::error::Result;
use crate::engine::Engine;
use crate::commands::Command;

pub mod help;
pub mod commands;
pub mod completions;

pub struct InternalCommand<'e> {
    pub name: &'static str,
    pub summary: &'static str,
    pub help: &'static str,
    pub args: Vec<String>,
    pub engine: &'e Engine,
    pub func: fn(&Engine, Vec<String>) -> Result<i32>,
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
