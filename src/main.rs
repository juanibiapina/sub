extern crate sub;

extern crate clap;

use clap::{Arg, Command, value_parser};

use std::path::PathBuf;
use std::process::exit;

use sub::engine::Engine;
use sub::error::Error;

fn main() {
    let app = init_cli();

    let matches = app.get_matches();

    let name = matches
        .get_one::<String>("name")
        .expect("`name` is required");
    let root = {
        let mut path = matches.get_one::<PathBuf>("bin").expect("`bin` is required").clone();
        path.pop(); // remove bin name
        if let Some(relative) = matches.get_one::<PathBuf>("relative") {
            path.push(relative)
        };
        path
    };
    let args = matches
        .get_many("commands")
        .map(|cmds| cmds.cloned().collect::<Vec<_>>())
        .unwrap_or_default();

    let xdg_dirs = match xdg::BaseDirectories::with_prefix(&name) {
        Ok(dir) => dir,
        Err(e) => {
            println!("Problem determining XDG base directory");
            println!("Original error: {}", e);
            exit(1);
        }
    };
    let cache_directory = match xdg_dirs.create_cache_directory("cache") {
        Ok(dir) => dir,
        Err(e) => {
            println!("Problem determining XDG cache directory");
            println!("Original error: {}", e);
            exit(1);
        }
    };

    let sub = Engine::new(name.clone(), root, cache_directory, args);

    match sub.run() {
        Ok(code) => exit(code),
        Err(Error::NoCompletions) => exit(1),
        Err(Error::SubCommandInterrupted) => exit(1),
        Err(Error::NonExecutable(_)) => exit(1),
        Err(Error::UnknownSubCommand(name)) => {
            sub.display_unknown_subcommand(&name);
            exit(1);
        }
    }
}

fn init_cli() -> Command {
    Command::new("sub")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("name")
                .long("name")
                .required(true)
                .help("Sets the binary name"),
        )
        .arg(
            Arg::new("bin")
                .long("bin")
                .required(true)
                .value_parser(value_parser!(PathBuf))
                .help("Sets the path of the CLI binary"),
        )
        .arg(
            Arg::new("relative")
                .long("relative")
                .value_parser(value_parser!(PathBuf))
                .help("Sets how to find the root directory based on the location of the bin"),
        )
        .arg(
            Arg::new("commands")
                .allow_hyphen_values(true)
                .trailing_var_arg(true)
                .num_args(..),
        )
}

#[test]
fn verify_cli() {
    init_cli().debug_assert();
}
