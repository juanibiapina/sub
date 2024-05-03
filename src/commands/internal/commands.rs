use std::path::Path;

use clap::{Arg, Command};

use crate::error::Result;
use crate::config::Config;
use crate::commands::internal;
use crate::commands::subcommand;

struct InternalCommandCommandsArgs {
    extension: Option<String>,

    names: Vec<String>,
}

fn parse_args(args: Vec<String>) -> InternalCommandCommandsArgs {
    let args = Command::new("commands")
        .no_binary_name(true)
        .arg(Arg::new("extension").short('e').long("extension"))
        .arg(Arg::new("names").num_args(1..))
        .get_matches_from(args);

    return InternalCommandCommandsArgs {
        extension: args.get_one::<String>("extension").cloned(),
        names: args.get_many("names").map(|s| s.cloned().collect::<Vec<String>>()).unwrap_or_default(),
    };
}

pub fn internal_commands(config: &Config, args: Vec<String>) -> internal::InternalCommand {
    internal::InternalCommand {
        name: "commands",
        summary: "List available commands",
        help: "",
        args,
        config,
        func: |config: &Config, args: Vec<String>| -> Result<i32> {
            let parsed_args = parse_args(args.clone());

            for subcommand in subcommand(config, parsed_args.names)?.subcommands() {
                // If an extension is provided, only show subcommands with that extension
                match parsed_args.extension {
                    Some(ref extension) => {
                        if let Some(subcommand_extension) = Path::new(subcommand.name()).extension() {
                            if *subcommand_extension == **extension {
                                println!("{}", subcommand.name());
                            }
                        }
                    },
                    None => {
                        println!("{}", subcommand.name());
                    },
                }
            }

            Ok(0)
        },
    }
}

