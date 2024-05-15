use std::process::exit;
use std::path::PathBuf;

use clap::{Command, ColorChoice};
use clap::builder::Styles;

#[derive(Clone)]
pub enum Color {
    Auto,
    Always,
    Never,
}

#[derive(Clone)]
pub struct Config {
    pub name: String,
    pub color: Color,
    pub root: PathBuf,
    pub cache_directory: PathBuf,
}

impl Config {
    pub fn new(name: String, root: PathBuf, color: Color) -> Config {
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

        Command::new(name.to_owned()).color(color_choice).styles(styles)
    }
}
