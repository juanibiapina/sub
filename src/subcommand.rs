use std::fs::DirEntry;
use std::os::unix::fs::PermissionsExt;

use crate::parser;

pub enum SubCommand {
    InternalCommand(InternalCommand),
    ExternalCommand(ExternalCommand),
}

impl SubCommand {
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

        Some(SubCommand::ExternalCommand(ExternalCommand {
            name,
            summary,
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
        }
    }

    pub fn summary(&self) -> &str {
        match self {
            SubCommand::InternalCommand(c) => &c.summary,
            SubCommand::ExternalCommand(c) => &c.summary,
        }
    }
}

pub struct InternalCommand {
    name: String,
    summary: String,
}

pub struct ExternalCommand {
    name: String,
    summary: String,
}
