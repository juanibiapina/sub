use crate::error::Result;
use crate::config::Config;
use crate::commands::internal;
use crate::commands::subcommand;

pub fn internal_help(config: &Config, args: Vec<String>) -> internal::InternalCommand {
    internal::InternalCommand {
        name: "help",
        summary: "Display help for a sub command",
        description: "A command is considered documented if it starts with a comment block
            that has a `Summary:' or `Usage:' section. Usage instructions can
            span multiple lines as long as subsequent lines are indented.
            The remainder of the comment block is displayed as extended
            documentation.",
            args,
            config,
            func: |config: &Config, args: Vec<String>| -> Result<i32> {
                let subcommand = subcommand(config, args.clone())?;

                println!("{}", subcommand.help());

                Ok(0)
            },
    }
}
