use crate::error::Result;

pub mod internal;
pub mod external;
pub mod toplevel;

pub trait Command {
    fn name(&self) -> &str;
    fn summary(&self) -> String;
    fn usage(&self) -> String;
    fn help(&self) -> String;
    fn subcommands(&self) -> Vec<Box<dyn Command + '_>>;
    fn completions(&self) -> Result<i32>;
    fn invoke(&self) -> Result<i32>;
}
