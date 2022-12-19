extern crate sub;

extern crate clap;

use clap::{Arg, Command};

use std::fs;
use std::process::exit;

use sub::engine::Engine;
use sub::error::Error;

fn main() {
    let app = init_cli();

    let matches = app.get_matches();

    let name = matches
        .get_one::<String>("name")
        .expect("`name` is required");
    let relative = matches.get_one::<String>("relative");
    let bin = matches.get_one::<String>("bin").expect("`bin` is required");
    let root = match fs::canonicalize(&bin) {
        Ok(mut path) => {
            path.pop(); // remove bin name
            path.push(relative.map_or(".", |str| str.as_str()));
            match fs::canonicalize(&path) {
                Ok(path) => path,
                Err(e) => {
                    println!("Invalid root path: {}", path.as_path().to_str().unwrap());
                    println!("Original error: {}", e);
                    exit(1)
                }
            }
        }
        Err(e) => {
            println!("Invalid bin path: {}", bin);
            println!("Original error: {}", e);
            exit(1)
        }
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
                .help("Sets the path of the CLI binary"),
        )
        .arg(
            Arg::new("relative")
                .long("relative")
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
