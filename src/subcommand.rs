use std::fs::DirEntry;
use std::os::unix::fs::PermissionsExt;

use crate::parser;

pub struct SubCommand {
    name: String,
    summary: String,
}

impl SubCommand {
    pub fn new(name: String, summary: String) -> SubCommand {
        SubCommand {
            name,
            summary,
        }
    }

    pub fn from_entry(entry: &DirEntry) -> Option<SubCommand> {
        let name = entry.file_name().into_string().unwrap();

        if name.starts_with('.') {
            return None;
        }

        if entry.path().is_dir() {
            return None;
        }

        if entry.metadata().unwrap().permissions().mode() & 0o111 == 0 {
            return None;
        }

        let summary = parser::extract_summary(&entry.path());

        Some(SubCommand::new(
                name,
                summary,
        ))
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn summary(&self) -> &str {
        &self.summary
    }
}
