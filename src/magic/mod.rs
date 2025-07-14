pub mod error;
pub mod help_parser;
pub mod metadata;
pub mod usage_parser;
pub mod option_parser;
pub mod completion;
pub mod script;

pub use error::{MagicError, Result};
pub use help_parser::{extract_initial_comment_block, extract_docs, provides_completions};
pub use metadata::{Docs, Metadata};
pub use usage_parser::{usage_parser, UsageLang, ArgSpec, ArgBase, parse_usage_line};
pub use option_parser::{option_parser, OptionSpec, parse_option_line};
pub use completion::{CompletionInfo, CompletionType};
pub use script::{Script, parse_script, extract_help, extract_usage, extract_options};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    
    #[test]
    fn test_magic_module_integration() {
        // Create a test script file
        let test_script_content = r#"#!/usr/bin/env bash
#
# Summary: Test script for magic module
#
# Usage: {cmd} <name> [--verbose] [--count=COUNT]
#
# Options:
#   name: The name to greet
#   verbose: Enable verbose output
#   count (script): Number of times to greet
#
# This is a test script for the magic module.
# It demonstrates how the magic module can parse
# script metadata and usage information.

echo "Hello, $1!"
"#;
        
        std::fs::write("/tmp/test_magic_script.sh", test_script_content).unwrap();
        
        let script_path = Path::new("/tmp/test_magic_script.sh");
        let script = parse_script(script_path).unwrap();
        
        // Test metadata extraction
        assert_eq!(script.metadata.summary, Some("Test script for magic module".to_string()));
        assert!(script.metadata.description.is_some());
        assert_eq!(script.metadata.provides_completions, false);
        
        // Test usage parsing
        assert!(script.usage.is_some());
        let usage = script.usage.as_ref().unwrap();
        assert_eq!(usage.arguments.len(), 3); // name, verbose, count
        assert_eq!(usage.rest, None);
        
        // Test options parsing
        assert_eq!(script.options.len(), 3);
        assert_eq!(script.options[0].name, "name");
        assert_eq!(script.options[1].name, "verbose");
        assert_eq!(script.options[2].name, "count");
        
        // Test completion info
        assert!(script.completion_info.provides_completions);
        assert!(script.completion_info.get_completion_type("count").is_some());
        
        // Clean up
        std::fs::remove_file("/tmp/test_magic_script.sh").unwrap();
    }
}