# Magic Module Documentation

## Overview

The `magic` module provides a clean API for extracting metadata, usage information, and completion details from individual script files. It was extracted from the `sub` codebase to isolate file-specific logic from CLI-specific concerns.

## Core Concepts

### Script Files
The magic module processes script files that contain special comments in their header:

```bash
#!/usr/bin/env bash
#
# Summary: Brief description of the script
#
# Usage: {cmd} <required_arg> [optional_arg] [--flag] [--option=VALUE]
#
# Options:
#   required_arg: Description of required argument
#   optional_arg: Description of optional argument
#   flag: Description of flag
#   option (script): Description of option with script completion
#
# Extended documentation goes here.
# This can be multiple lines and paragraphs.
```

### Main Types

#### `Script`
The primary type representing a parsed script file:

```rust
pub struct Script {
    pub path: PathBuf,
    pub metadata: Metadata,
    pub usage: Option<UsageLang>,
    pub options: Vec<OptionSpec>,
    pub completion_info: CompletionInfo,
}
```

#### `Metadata`
Contains basic script information:

```rust
pub struct Metadata {
    pub summary: Option<String>,
    pub description: Option<String>,
    pub provides_completions: bool,
}
```

#### `UsageLang`
Represents parsed usage syntax:

```rust
pub struct UsageLang {
    pub arguments: Vec<ArgSpec>,
    pub rest: Option<String>,
}
```

#### `ArgSpec`
Represents a single argument specification:

```rust
pub struct ArgSpec {
    pub base: ArgBase,
    pub required: bool,
    pub exclusive: bool,
}

pub enum ArgBase {
    Positional(String),
    Short(char),
    Long(String, Option<String>),
}
```

## Module Structure

### `magic::help_parser`
Extracts documentation from script comment blocks:

- `extract_initial_comment_block(path: &Path) -> Result<String>` - Reads initial comment block
- `extract_docs(path: &Path) -> Result<Docs>` - Parses structured documentation
- `provides_completions(path: &Path) -> Result<bool>` - Checks for completion capability

### `magic::usage_parser`
Parses usage comment syntax:

- `usage_parser()` - Parser combinator for usage syntax
- `parse_usage_line(line: &str) -> Result<UsageLang>` - Parse single usage line

### `magic::option_parser`
Parses option definitions:

- `option_parser()` - Parser combinator for option syntax
- `parse_option_line(line: &str) -> Result<OptionSpec>` - Parse single option line

### `magic::completion`
Handles completion types:

```rust
pub enum CompletionType {
    Script,                    // Script generates its own completions
    LiteralCommand(String),    // Run literal command for completions
}
```

### `magic::script`
High-level script abstraction:

- `Script::parse(path: &Path) -> Result<Script>` - Parse complete script
- `parse_script(path: &Path) -> Result<Script>` - Convenience function
- `extract_help(path: &Path) -> Result<Metadata>` - Extract only metadata
- `extract_usage(path: &Path) -> Result<Option<UsageLang>>` - Extract only usage
- `extract_options(path: &Path) -> Result<Vec<OptionSpec>>` - Extract only options

## Usage Examples

### Basic Script Parsing

```rust
use sub::magic::{parse_script, Script};
use std::path::Path;

let script_path = Path::new("my_script.sh");
let script = parse_script(script_path)?;

println!("Summary: {:?}", script.metadata.summary);
if let Some(usage) = &script.usage {
    println!("Arguments: {}", usage.arguments.len());
}
```

### Extract Only Metadata

```rust
use sub::magic::extract_help;
use std::path::Path;

let script_path = Path::new("my_script.sh");
let metadata = extract_help(script_path)?;

if let Some(summary) = &metadata.summary {
    println!("Script summary: {}", summary);
}
```

### Parse Usage Information

```rust
use sub::magic::extract_usage;
use std::path::Path;

let script_path = Path::new("my_script.sh");
if let Some(usage) = extract_usage(script_path)? {
    for arg in &usage.arguments {
        println!("Argument: {:?}", arg.base);
    }
}
```

## Usage Syntax

The magic module supports rich usage syntax:

### Basic Elements
- `{cmd}` - Placeholder for command name (required)
- `<required>` - Required positional argument
- `[optional]` - Optional positional argument
- `[-f]` - Optional short flag
- `[--flag]` - Optional long flag
- `[--option=VALUE]` - Optional long option with value
- `[--exclusive]!` - Exclusive option (cannot be used with others)
- `[rest]...` - Rest arguments (consumes remaining args)

### Examples
```bash
# Simple positional arguments
# Usage: {cmd} <name>

# Mixed arguments and flags
# Usage: {cmd} <input> [output] [-v] [--format=FORMAT]

# Rest arguments
# Usage: {cmd} <command> [args]...

# Exclusive options
# Usage: {cmd} [--json]! [--yaml]!
```

## Option Definitions

Options can specify completion behavior:

```bash
# Options:
#   file: Simple file argument
#   format (script): Script provides completions
#   mode (`echo json yaml xml`): Command provides completions
```

### Completion Types
- **No type**: No completions provided
- **`script`**: Script generates completions when called with special environment variables
- **`` `command` ``**: Run literal command to generate completions

## Error Handling

The magic module defines its own error types:

```rust
pub enum MagicError {
    InvalidUsageString(String),
    InvalidOptionString(String),
    InvalidUTF8,
    IoError(String),
    ParseError(String),
}
```

Errors are returned as `Result<T, MagicError>` and can be handled appropriately by the calling code.

## Integration with Sub

The magic module is designed to be used by the `sub` CLI tool but is decoupled from sub-specific concerns:

- **File processing**: Magic handles all file parsing
- **CLI integration**: Sub handles clap integration and command building
- **Process execution**: Sub handles script execution and environment setup
- **Configuration**: Sub handles CLI-specific configuration

## Testing

The magic module includes comprehensive tests:

```rust
// Test usage parsing
#[test]
fn test_usage_parsing() {
    let input = "# Usage: {cmd} <name> [--verbose]";
    let result = parse_usage_line(input).unwrap();
    assert_eq!(result.arguments.len(), 2);
}

// Test full script parsing
#[test]
fn test_script_parsing() {
    let script = parse_script(Path::new("test_script.sh")).unwrap();
    assert!(script.metadata.summary.is_some());
}
```

## Design Principles

1. **Separation of Concerns**: Pure file processing logic separated from CLI concerns
2. **Reusability**: Can be used by other tools beyond `sub`
3. **Error Handling**: Comprehensive error types for different failure modes
4. **Parser Combinators**: Uses chumsky for robust parsing
5. **Minimal Dependencies**: Only depends on essential crates

## Future Enhancements

Potential areas for future development:

1. **Language Support**: Support for scripts in other languages
2. **Validation**: Enhanced validation of usage syntax
3. **Performance**: Caching of parsed results
4. **Documentation**: Generate documentation from scripts
5. **IDE Integration**: Language server protocol support