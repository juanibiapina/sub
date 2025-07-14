use std::path::{Path, PathBuf};
use std::collections::HashMap;

use super::error::Result;
use super::help_parser::{extract_docs, provides_completions};
use super::metadata::Metadata;
use super::usage_parser::{parse_usage_line, UsageLang};
use super::option_parser::{parse_option_line, OptionSpec};
use super::completion::CompletionInfo;

#[derive(Debug, Clone)]
pub struct Script {
    pub path: PathBuf,
    pub metadata: Metadata,
    pub usage: Option<UsageLang>,
    pub options: Vec<OptionSpec>,
    pub completion_info: CompletionInfo,
}

impl Script {
    pub fn parse(path: &Path) -> Result<Script> {
        let docs = extract_docs(path)?;
        let provides_completions_flag = provides_completions(path)?;
        
        let metadata = Metadata::from_docs(&docs, provides_completions_flag);
        
        let usage = if let Some(usage_line) = &docs.usage {
            Some(parse_usage_line(usage_line)?)
        } else {
            None
        };
        
        let mut options = Vec::new();
        let mut completion_types = HashMap::new();
        
        for option_line in &docs.options {
            match parse_option_line(option_line) {
                Ok(option) => {
                    if let Some(completion_type) = &option.completion_type {
                        completion_types.insert(option.name.clone(), completion_type.clone());
                    }
                    options.push(option);
                },
                Err(e) => {
                    // For now, we'll skip invalid options but could collect errors
                    eprintln!("Warning: Failed to parse option '{}': {}", option_line, e);
                }
            }
        }
        
        let completion_info = CompletionInfo::with_completions(completion_types);
        
        Ok(Script {
            path: path.to_path_buf(),
            metadata,
            usage,
            options,
            completion_info,
        })
    }
    
    pub fn has_usage(&self) -> bool {
        self.usage.is_some()
    }
    
    pub fn provides_completions(&self) -> bool {
        self.completion_info.provides_completions
    }
}

// Convenience functions for backward compatibility
pub fn parse_script(path: &Path) -> Result<Script> {
    Script::parse(path)
}

pub fn extract_help(path: &Path) -> Result<Metadata> {
    let docs = extract_docs(path)?;
    let provides_completions_flag = provides_completions(path)?;
    Ok(Metadata::from_docs(&docs, provides_completions_flag))
}

pub fn extract_usage(path: &Path) -> Result<Option<UsageLang>> {
    let docs = extract_docs(path)?;
    if let Some(usage_line) = &docs.usage {
        Ok(Some(parse_usage_line(usage_line)?))
    } else {
        Ok(None)
    }
}

pub fn extract_options(path: &Path) -> Result<Vec<OptionSpec>> {
    let docs = extract_docs(path)?;
    let mut options = Vec::new();
    
    for option_line in &docs.options {
        match parse_option_line(option_line) {
            Ok(option) => options.push(option),
            Err(e) => {
                eprintln!("Warning: Failed to parse option '{}': {}", option_line, e);
            }
        }
    }
    
    Ok(options)
}