extern crate sub;

extern crate clap;

use clap::{Args, Parser};

use std::path::{Path, PathBuf};
use std::process::exit;

use sub::commands::subcommand;
use sub::config::{Color, Config};
use sub::error::Error;

fn main() {
    let (config, cliargs) = parse_sub_cli_args();

    let user_cli_args = parse_user_cli_args(&config, cliargs);

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
        UserCliMode::Validate => {
            let errors = subcommand.validate();
            for error in &errors {
                println!("{}: {}", error.0.display(), print_error(error.1.clone()));
            }

            if errors.is_empty() {
                exit(0);
            } else {
                exit(1);
            }
        }
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
        Error::NoLibexecDir => "libexec directory not found in root".to_string(),
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
        Error::NoLibexecDir => {
            if !silent {
                println!("{}: libexec directory not found in root", config.name);
            }
            exit(1);
        }
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct SubCli {
    #[arg(short, long)]
    #[arg(default_value = "auto")]
    #[arg(help = "Enable colored output for help messages")]
    color: Color,

    #[arg(long)]
    #[arg(help = "Allow partial matches of long arguments")]
    infer_long_arguments: bool,

    #[arg(long)]
    #[arg(help = "Sets the CLI name - used in help and error messages")]
    name: String,

    #[command(flatten)]
    path_args: PathArgs,

    #[arg(help = "Arguments to pass to the CLI", raw = true)]
    cliargs: Vec<String>,
}

#[derive(Args)]
#[group(id = "path", multiple = true, required = true)]
struct PathArgs {
    #[arg(long)]
    #[arg(value_parser = absolute_path)]
    #[arg(conflicts_with_all = ["executable", "relative"])]
    #[arg(help = "Absolute path to the CLI root directory (where libexec lives)")]
    absolute: Option<PathBuf>,

    #[arg(long)]
    #[arg(requires = "relative")]
    #[arg(help = "Sets the path of the CLI executable; only use in combination with --relative")]
    executable: Option<PathBuf>,

    #[arg(long)]
    #[arg(requires = "executable")]
    #[arg(help = "Sets how to find the root directory based on the location of the executable; Only use in combination with --executable")]
    relative: Option<PathBuf>,
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    SubCli::command().debug_assert();
}

#[derive(PartialEq)]
enum UserCliMode {
    Invoke,
    Usage,
    Help,
    Commands(Option<String>),
    Completions,
    Validate,
}

struct UserCliArgs {
    mode: UserCliMode,
    commands_with_args: Vec<String>,
}

fn parse_user_cli_args(config: &Config, cliargs: Vec<String>) -> UserCliArgs {
    let user_cli_command = config.user_cli_command(&config.name);

    let args = match user_cli_command.clone().try_get_matches_from(cliargs) {
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
        } else if args.get_one::<bool>("validate").cloned().unwrap_or(false) {
            UserCliMode::Validate
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

fn parse_sub_cli_args() -> (Config, Vec<String>) {
    let args = SubCli::parse();

    let root = match args.path_args.absolute {
        Some(path) => path.clone(),
        None => {
            let mut path = args.path_args.executable
                // this code is unreachable because clap is validating the arguments
                .unwrap_or_else(|| unreachable!("Missing `executable` argument"))
                .canonicalize()
                .expect("Invalid `executable` path")
                .clone();

            path.pop(); // remove executable name

            // this code is unreachable because clap is validating the arguments
            let relative = args.path_args.relative.unwrap_or_else(|| unreachable!("Missing `relative` argument"));
            path.push(relative);

            path.canonicalize().expect("Invalid `executable` or `relative` arguments")
        }
    };

    let config = Config::new(args.name, root, args.color, args.infer_long_arguments);

    return (config, args.cliargs);
}

fn absolute_path(s: &str) -> Result<PathBuf, String> {
    let path = Path::new(s);
    if path.is_absolute() {
        Ok(path.to_owned())
    } else {
        Err("not an absolute path".to_string())
    }
}
