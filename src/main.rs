extern crate sub;

extern crate clap;

use clap::{value_parser, Arg, ArgAction, ArgGroup, Command};

use std::path::{Path, PathBuf};
use std::process::exit;

use sub::commands::subcommand;
use sub::config::{Color, Config};
use sub::error::Error;

fn main() {
    let sub_cli_args = parse_sub_cli_args();

    let config = Config::new(sub_cli_args.name, sub_cli_args.root, sub_cli_args.color, sub_cli_args.infer_long_arguments);

    if sub_cli_args.validate {
        let top_level_command = match subcommand(&config, Vec::new()) {
            Ok(subcommand) => subcommand,
            Err(error) => handle_error(
                &config,
                error,
                false,
            ),
        };

        let errors = top_level_command.validate();
        for error in &errors {
            println!("{}: {}", error.0.display(), print_error(error.1.clone()));
        }

        if errors.is_empty() {
            exit(0);
        } else {
            exit(1);
        }
    }

    let user_cli_command = config.user_cli_command(&config.name);
    let user_cli_args = parse_user_cli_args(&user_cli_command, sub_cli_args.cliargs);

    let subcommand = match subcommand(&config, user_cli_args.commands_with_args.clone()) {
        Ok(subcommand) => subcommand,
        Err(error) => handle_error(
            &config,
            error,
            user_cli_args.mode == UserCliMode::Completions,
        ),
    };

    match user_cli_args.mode {
        UserCliMode::Invoke => match subcommand.invoke() {
            Ok(code) => exit(code),
            Err(error) => handle_error(&config, error, false),
        },
        UserCliMode::Usage => {
            let usage = match subcommand.usage() {
                Ok(usage) => usage,
                Err(error) => handle_error(&config, error, false),
            };

            println!("{}", usage);
        }
        UserCliMode::Help => {
            let help = match subcommand.help() {
                Ok(help) => help,
                Err(error) => handle_error(&config, error, false),
            };

            println!("{}", help);
        }
        UserCliMode::Commands(extension) => {
            for subcommand in subcommand.subcommands() {
                if let Some(extension) = &extension {
                    if let Some(subcommand_extension) = Path::new(subcommand.name()).extension() {
                        if subcommand_extension == extension.as_str() {
                            println!("{}", subcommand.name());
                        }
                    }
                } else {
                    println!("{}", subcommand.name());
                }
            }
        }
        UserCliMode::Completions => match subcommand.completions() {
            Ok(code) => exit(code),
            Err(error) => handle_error(&config, error, true),
        },
    }
}

fn print_error(error: Error) -> String {
    match error {
        Error::NoCompletions => "no completions".to_string(),
        Error::SubCommandInterrupted => "sub command interrupted".to_string(),
        Error::NonExecutable(_) => "non-executable".to_string(),
        Error::UnknownSubCommand(name) => format!("unknown sub command '{}'", name),
        Error::InvalidUsageString(errors) => {
            let mut message = "invalid usage string".to_string();
            for error in errors {
                message.push_str(&format!("\n  {}", error));
            }
            message
        }
        Error::InvalidUTF8 => "invalid UTF-8".to_string(),
    }
}

fn handle_error(config: &Config, error: Error, silent: bool) -> ! {
    match error {
        Error::NoCompletions => exit(1),
        Error::SubCommandInterrupted => exit(1),
        Error::NonExecutable(_) => exit(1),
        Error::UnknownSubCommand(name) => {
            if !silent {
                println!("{}: no such sub command '{}'", config.name, name);
            }
            exit(1);
        }
        Error::InvalidUsageString(errors) => {
            if !silent {
                println!("{}: invalid usage string", config.name);
                for error in errors {
                    println!("  {}", error);
                }
            }
            exit(1);
        }
        Error::InvalidUTF8 => {
            if !silent {
                println!("invalid UTF-8");
            }
            exit(1);
        }
    }
}

fn init_sub_cli() -> Command {
    Command::new("sub")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Dynamically generate rich CLIs from scripts.")
        .arg(
            Arg::new("color")
                .long("color")
                .value_name("WHEN")
                .value_parser(["auto", "always", "never"])
                .default_value("auto")
                .num_args(1)
                .help("Enable colored output for help messages"),
        )
        .arg(
            Arg::new("infer-long-arguments")
                .long("infer-long-arguments")
                .num_args(0)
                .help("Allow partial matches of long arguments"),
        )
        .arg(
            Arg::new("name")
                .long("name")
                .required(true)
                .help("Sets the CLI name. Used in help and error messages."),
        )
        .arg(
            Arg::new("bin")
                .long("bin")
                .value_parser(value_parser!(PathBuf))
                .help("Sets the path of the CLI binary. Only use in combination with --relative."),
        )
        .arg(
            Arg::new("relative")
                .long("relative")
                .value_parser(value_parser!(PathBuf))
                .help("Sets how to find the root directory based on the location of the bin. Only use in combination with --bin."),
        )
        .arg(
            Arg::new("absolute")
                .long("absolute")
                .required(true)
                .value_name("PATH")
                .value_parser(absolute_path)
                .help("Absolute path to the CLI root directory (where libexec lives)"),
        )
        .arg(
            Arg::new("validate")
                .long("validate")
                .num_args(0)
                .action(ArgAction::SetTrue)
                .help("Validate that the CLI is correctly configured"),
        )
        .group(
            ArgGroup::new("bin_and_relative")
                .args(["bin", "relative"])
                .multiple(true)
                .conflicts_with("absolute"),
        )
        .arg(
            Arg::new("cliargs")
                .raw(true)
                .help("Arguments to pass to the CLI"),

        )
}

#[test]
fn verify_cli() {
    init_sub_cli().debug_assert();
}

struct SubCliArgs {
    name: String,
    color: Color,
    root: PathBuf,
    infer_long_arguments: bool,
    validate: bool,
    cliargs: Vec<String>,
}

#[derive(PartialEq)]
enum UserCliMode {
    Invoke,
    Usage,
    Help,
    Commands(Option<String>),
    Completions,
}

struct UserCliArgs {
    mode: UserCliMode,
    commands_with_args: Vec<String>,
}

fn parse_user_cli_args(cmd: &Command, cliargs: Vec<String>) -> UserCliArgs {
    let args = match cmd.clone().try_get_matches_from(cliargs) {
        Ok(args) => args,
        Err(e) => {
            e.print().unwrap();
            exit(1);
        }
    };

    UserCliArgs {
        mode: if args.get_one::<bool>("usage").cloned().unwrap_or(false) {
            UserCliMode::Usage
        } else if args.get_one::<bool>("help").cloned().unwrap_or(false) {
            UserCliMode::Help
        } else if args.get_one::<bool>("commands").cloned().unwrap_or(false) {
            UserCliMode::Commands(args.get_one::<String>("extension").cloned())
        } else if args
            .get_one::<bool>("completions")
            .cloned()
            .unwrap_or(false)
        {
            UserCliMode::Completions
        } else {
            UserCliMode::Invoke
        },
        commands_with_args: args
            .get_many("commands_with_args")
            .map(|cmds| cmds.cloned().collect::<Vec<_>>())
            .unwrap_or_default(),
    }
}

fn parse_sub_cli_args() -> SubCliArgs {
    let sub_cli = init_sub_cli();
    let args = sub_cli.get_matches();

    SubCliArgs {
        name: args
            .get_one::<String>("name")
            .expect("`name` is mandatory")
            .clone(),

        color: match args
            .get_one::<String>("color")
            .expect("`color` is mandatory")
            .clone()
            .as_ref()
        {
            "auto" => Color::Auto,
            "always" => Color::Always,
            "never" => Color::Never,
            _ => unreachable!(),
        },

        infer_long_arguments: args.get_one::<bool>("infer-long-arguments").cloned().unwrap_or(false),

        validate: args.get_flag("validate"),

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
