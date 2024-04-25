use std::path::PathBuf;

#[derive(Clone)]
pub struct Config {
    pub name: String,
    pub root: PathBuf,
    pub cache_directory: PathBuf,
}

impl Config {
    pub fn libexec_path(&self) -> PathBuf {
        let mut path = self.root.clone();
        path.push("libexec");
        return path;
    }
}
