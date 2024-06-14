use std::process::exit;
use std::path::PathBuf;

use clap::{Command, ColorChoice, Arg, ArgGroup, ValueEnum};
use clap::builder::Styles;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Color {
    /// Enables colored output only when the output is going to a terminal or TTY.
    Auto,
    /// Enables colored output regardless of whether or not the output is going to a terminal/TTY.
    Always,
    /// Disables colored output no matter if the output is going to a terminal/TTY, or not.
    Never,
}

#[derive(Clone)]
pub struct Config {
    pub name: String,
    pub color: Color,
    pub root: PathBuf,
    infer_long_arguments: bool,
    pub cache_directory: PathBuf,
}

impl Config {
    pub fn new(name: String, root: PathBuf, color: Color, infer_long_arguments: bool) -> Config {
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

        Config {
            name,
            color,
            infer_long_arguments,
            root,
            cache_directory,
        }
    }

    pub fn libexec_path(&self) -> PathBuf {
        let mut path = self.root.clone();
        path.push("libexec");
        return path;
    }

    pub fn base_command(&self, name: &str) -> Command {
        let color_choice = match self.color {
            Color::Auto => ColorChoice::Auto,
            Color::Always => ColorChoice::Always,
            Color::Never => ColorChoice::Never,
        };

        let styles = match self.color {
            Color::Auto => Styles::default(),
            Color::Always => Styles::default(),
            Color::Never => Styles::plain(),
        };

        Command::new(name.to_owned()).color(color_choice).styles(styles).infer_long_args(self.infer_long_arguments)
    }

    pub fn user_cli_command(&self, name: &str) -> Command {
        self.base_command(name).no_binary_name(true).disable_help_flag(true)
            .arg(Arg::new("usage").long("usage").num_args(0).help("Print usage"))
            .arg(Arg::new("help").short('h').long("help").num_args(0).help("Print help"))
            .arg(Arg::new("completions").long("completions").num_args(0).help("Print completions"))
            .arg(Arg::new("validate").long("validate").num_args(0).help("Validate subcommand"))

            .arg(Arg::new("commands").long("commands").num_args(0).help("Print subcommands"))
            .arg(Arg::new("extension").long("extension").num_args(1).help("Filter subcommands by extension"))
            .group(ArgGroup::new("extension_group").args(["extension"]).requires("commands"))

            .group(ArgGroup::new("exclusion").args(["commands", "completions", "usage", "help", "validate"]).multiple(false).required(false))

            .arg(Arg::new("commands_with_args").trailing_var_arg(true).allow_hyphen_values(true).num_args(..))
    }
}
