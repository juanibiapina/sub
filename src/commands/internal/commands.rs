use crate::error::Result;
use crate::engine::Engine;
use crate::commands::internal;

pub fn internal_commands(engine: &Engine, args: Vec<String>) -> internal::InternalCommand {
    internal::InternalCommand {
        name: "commands",
        summary: "List available commands",
        help: "",
        args,
        engine,
        func: |engine: &Engine, args: Vec<String>| -> Result<i32> {
            for subcommand in engine.subcommands(args) {
                println!("{}", subcommand.name());
            }

            Ok(0)
        },
    }
}

