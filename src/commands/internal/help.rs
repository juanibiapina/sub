use crate::error::Result;
use crate::engine::Engine;
use crate::commands::internal;

pub fn internal_help(engine: &Engine, args: Vec<String>) -> internal::InternalCommand {
    internal::InternalCommand {
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

                let subcommands = subcommand.subcommands();
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
