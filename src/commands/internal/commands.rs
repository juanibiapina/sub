use crate::error::Result;
use crate::config::Config;
use crate::commands::internal;
use crate::commands::subcommand;

pub fn internal_commands(config: &Config, args: Vec<String>) -> internal::InternalCommand {
    internal::InternalCommand {
        name: "commands",
        summary: "List available commands",
        help: "",
        args,
        config,
        func: |config: &Config, args: Vec<String>| -> Result<i32> {
            for subcommand in subcommand(config, args)?.subcommands() {
                println!("{}", subcommand.name());
            }

            Ok(0)
        },
    }
}

