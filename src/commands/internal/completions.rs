use crate::error::Result;
use crate::engine::Engine;
use crate::commands::internal;

pub fn internal_completions(engine: &Engine, args: Vec<String>) -> internal::InternalCommand {
    internal::InternalCommand {
        name: "completions",
        summary: "List completions for a sub command",
        help: "",
        args,
        engine,
        func: |engine: &Engine, args: Vec<String>| -> Result<i32> {
            if let Ok(subcommand) = engine.subcommand(args) {
                subcommand.completions()
            } else {
                Ok(1)
            }
        },
    }
}

