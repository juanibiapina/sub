extern crate sub;

extern crate clap;

use clap::{value_parser, Arg, ArgGroup, Command};

use std::path::{Path, PathBuf};
use std::process::exit;

use sub::commands::subcommand;
use sub::config::Config;
use sub::error::Error;

fn main() {
    let sub_cli_args = parse_sub_cli_args();

    let xdg_dirs = match xdg::BaseDirectories::with_prefix(&sub_cli_args.name) {
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

    let config = Config {
        name: sub_cli_args.name,
        root: sub_cli_args.root,
        cache_directory,
    };

    let user_cli_command = Command::new(&config.name).no_binary_name(true).disable_help_flag(true)
        .arg(Arg::new("usage").long("usage").num_args(0).help("Print usage"))
        .arg(Arg::new("help").short('h').long("help").num_args(0).help("Print help"))
        .group(ArgGroup::new("help_group").args(["usage", "help"]).multiple(false).required(false))
        .arg(Arg::new("commands_with_args").trailing_var_arg(true).allow_hyphen_values(true).num_args(..));

    let args = match user_cli_command.try_get_matches_from(sub_cli_args.cliargs) {
        Ok(args) => args,
        Err(e) => {
            e.print().unwrap();
            exit(1);
        }
    };

    let commands_with_args = args.get_many::<String>("commands_with_args").map(|cmds| cmds.cloned().collect::<Vec<_>>()).unwrap_or_default();

    let subcommand = match subcommand(&config, commands_with_args) {
        Ok(subcommand) => subcommand,
        Err(error) => handle_error(&config, error),
    };

    if args.get_one::<bool>("usage").cloned().unwrap_or(false) {
        println!("{}", subcommand.usage());
        exit(0);
    }

    if args.get_one::<bool>("help").cloned().unwrap_or(false) {
        println!("{}", subcommand.help());
        exit(0);
    }

    match subcommand.invoke() {
        Ok(code) => exit(code),
        Err(error) => handle_error(&config, error),
    }
}

pub fn display_unknown_subcommand(config: &Config, name: &str) {
    println!("{}: no such sub command '{}'", config.name, name);
}

fn display_invalid_usage_string(config: &Config, errors: &[chumsky::prelude::Simple<char>]) {
    println!("{}: invalid usage string", config.name);
    for error in errors {
        println!("  {}", error);
    }
}

struct SubCliArgs {
    name: String,
    root: PathBuf,
    cliargs: Vec<String>,
}

fn handle_error(config: &Config, error: Error) -> ! {
    match error {
        Error::NoCompletions => exit(1),
        Error::SubCommandInterrupted => exit(1),
        Error::NonExecutable(_) => exit(1),
        Error::UnknownSubCommand(name) => {
            display_unknown_subcommand(config, &name);
            exit(1);
        }
        Error::InvalidUsageString(errors) => {
            display_invalid_usage_string(config, &errors);
            exit(1);
        }
    }
}

fn init_sub_cli() -> Command {
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
                .conflicts_with("absolute")
                .help("Sets how to find the root directory based on the location of the bin"),
        )
        .arg(
            Arg::new("absolute")
                .long("absolute")
                .value_parser(absolute_path)
                .help("Sets how to find the root directory as an absolute path"),
        )
        .group(
            ArgGroup::new("path")
                .args(["bin", "absolute"])
                .required(true),
        )
        .arg(
            Arg::new("cliargs").raw(true),
        )
}

#[test]
fn verify_cli() {
    init_sub_cli().debug_assert();
}

fn parse_sub_cli_args() -> SubCliArgs {
    let sub_cli = init_sub_cli();
    let args = sub_cli.get_matches();

    SubCliArgs {
        name: args
            .get_one::<String>("name")
            .expect("`name` is mandatory")
            .clone(),

        cliargs: args
            .get_many("cliargs")
            .map(|cmds| cmds.cloned().collect::<Vec<_>>())
            .unwrap_or_default(),

        root: match args.get_one::<PathBuf>("absolute") {
            Some(path) => path.clone(),
            None => {
                let mut path = args
                    .get_one::<PathBuf>("bin")
                    .expect("Either `bin` or `absolute` is required")
                    .canonicalize()
                    .expect("Invalid `bin` path")
                    .clone();
                path.pop(); // remove bin name
                if let Some(relative) = args.get_one::<PathBuf>("relative") {
                    path.push(relative)
                };
                path.canonicalize()
                    .expect("Invalid `bin` or `relative` arguments")
            }
        },
    }
}

fn absolute_path(s: &str) -> Result<PathBuf, String> {
    let path = Path::new(s);
    if path.is_absolute() {
        Ok(path.to_owned())
    } else {
        Err("not an absolute path".to_string())
    }
}
