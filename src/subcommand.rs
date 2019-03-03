use std::fs::DirEntry;
use std::os::unix::fs::PermissionsExt;

use crate::parser;

pub struct SubCommand {
    pub name: String,
    pub summary: String,
    pub usage: String,
    pub help: String,
}

impl SubCommand {
    pub fn new(name: String, summary: String, usage: String, help: String) -> SubCommand {
        SubCommand {
            name,
            summary,
            usage,
            help,
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

        let name = entry.file_name().into_string().unwrap();

        let summary = parser::extract_summary(&entry.path());
        let usage = parser::extract_usage(&entry.path());
        let help = parser::extract_help(&entry.path());

        Some(SubCommand::new(
                name,
                summary,
                usage,
                help
        ))
    }
}
