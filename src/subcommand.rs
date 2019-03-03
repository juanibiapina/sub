use std::fs::DirEntry;
use std::path::Path;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::parser;

pub enum SubCommand {
    InternalCommand(InternalCommand),
    ExternalCommand(ExternalCommand),
    NestedCommand(NestedCommand),
}

impl SubCommand {
    fn from_dir(path: &Path) -> Option<SubCommand> {
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();

        Some(SubCommand::NestedCommand(NestedCommand {
            name,
            path: path.to_owned(),
        }))
    }

    pub fn from_entry(entry: &DirEntry) -> Option<SubCommand> {
        let name = entry.file_name().into_string().unwrap();

        if name.starts_with('.') {
            return None;
        }

        if entry.path().is_dir() {
            return SubCommand::from_dir(&entry.path());
        }

        if entry.metadata().unwrap().permissions().mode() & 0o111 == 0 {
            return None;
        }

        let path = entry.path();

        Some(SubCommand::ExternalCommand(ExternalCommand {
            name,
            path,
        }))
    }

    pub fn internal(name: String, summary: String) -> SubCommand {
        SubCommand::InternalCommand(InternalCommand{
            name,
            summary,
        })
    }

    pub fn name(&self) -> &str {
        match self {
            SubCommand::InternalCommand(c) => &c.name,
            SubCommand::ExternalCommand(c) => &c.name,
            SubCommand::NestedCommand(c) => &c.name,
        }
    }

    pub fn summary(&self) -> String {
        match self {
            SubCommand::InternalCommand(c) => c.summary.clone(),
            SubCommand::ExternalCommand(c) => parser::extract_summary(&c.path),
            SubCommand::NestedCommand(c) => {
                let mut readme_path = c.path.clone();
                readme_path.push("README");

                if readme_path.exists() {
                    parser::extract_summary(&readme_path)
                } else {
                    "".to_owned()
                }
            },
        }
    }
}

pub struct InternalCommand {
    name: String,
    summary: String,
}

pub struct ExternalCommand {
    name: String,
    path: PathBuf,
}

pub struct NestedCommand {
    name: String,
    path: PathBuf,
}
