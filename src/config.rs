use std::path::PathBuf;

#[derive(Clone)]
pub struct Config {
    pub name: String,
    pub root: PathBuf,
    pub cache_directory: PathBuf,
}
