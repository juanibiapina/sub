use crate::error::Result;
use crate::commands::internal;
use crate::config::Config;
use crate::commands::subcommand;

pub fn internal_completions(config: &Config, args: Vec<String>) -> internal::InternalCommand {
    internal::InternalCommand {
        name: "completions",
        summary: "List completions for a sub command",
        description: "",
        args,
        config,
        func: |config: &Config, args: Vec<String>| -> Result<i32> {
            if let Ok(subcommand) = subcommand(config, args) {
                subcommand.completions()
            } else {
                Ok(1)
            }
        },
    }
}

