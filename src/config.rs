use std::process::exit;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Config {
    pub name: String,
    pub root: PathBuf,
    pub cache_directory: PathBuf,
}

impl Config {
    pub fn new(name: String, root: PathBuf) -> Config {
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
            root,
            cache_directory,
        }
    }

    pub fn libexec_path(&self) -> PathBuf {
        let mut path = self.root.clone();
        path.push("libexec");
        return path;
    }
}
