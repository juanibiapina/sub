extern crate sub;

extern crate clap;

use clap::{App, AppSettings, Arg};

use std::fs;
use std::process::exit;

use sub::engine::Engine;
use sub::error::Error;

fn main() {
    let app = init_cli();

    let matches = app.get_matches();

    let name = matches.value_of("name").unwrap().to_owned();
    let relative = matches.value_of("relative").unwrap_or(".");
    let bin = matches.value_of("bin").unwrap();
    let root = match fs::canonicalize(&bin) {
        Ok(mut path) => {
            path.pop(); // remove bin name
            path.push(&relative);
            match fs::canonicalize(&path) {
                Ok(path) => path,
                Err(e) => {
                    println!("Invalid root path: {}", path.as_path().to_str().unwrap());
                    println!("Original error: {}", e);
                    exit(1)
                }
            }
        },
        Err(e) => {
            println!("Invalid bin path: {}", matches.value_of("bin").unwrap());
            println!("Original error: {}", e);
            exit(1)
        }
    };
    let args = matches
        .values_of("commands")
        .and_then(|args| Some(args.map(|s| s.to_owned()).collect::<Vec<_>>()))
        .unwrap_or_default();

    let sub = Engine::new(name, root, args);

    match sub.run() {
        Ok(code) => exit(code),
        Err(Error::NoSubCommand) => {
            sub.display_help();
            exit(0);
        },
        Err(Error::NoCompletions) => exit(1),
        Err(Error::SubCommandInterrupted) => exit(1),
        Err(Error::NonExecutable(_)) => exit(1),
        Err(Error::UnknownSubCommand(name)) => {
            sub.display_unknown_subcommand(&name);
            exit(1);
        },
    }
}

fn init_cli<'a, 'b>() -> App<'a, 'b> {
    App::new("sub")
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::AllowLeadingHyphen)
        .setting(AppSettings::TrailingVarArg)
        .arg(Arg::with_name("name")
             .long("name")
             .required(true)
             .takes_value(true)
             .help("Sets the binary name"))
        .arg(Arg::with_name("bin")
             .long("bin")
             .required(true)
             .takes_value(true)
             .help("Sets the path of the CLI binary"))
        .arg(Arg::with_name("relative")
             .long("relative")
             .takes_value(true)
             .help("Sets how to find the root directory based on the location of the bin"))
        .arg(Arg::with_name("commands")
             .allow_hyphen_values(true)
             .last(true)
             .multiple(true))
}
