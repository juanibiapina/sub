# Magic Module Extraction Audit Report

## Executive Summary
This report audits the `sub` codebase to identify all file-specific logic that should be extracted into a new `magic` module for processing individual script files.

## Current Architecture Overview
The `sub` tool generates dynamic CLIs from script directories. Key components:
- **Main CLI**: Parses `sub` arguments and delegates to subcommands
- **Commands**: File and directory command implementations
- **File Processing**: Parsing script comments for help, usage, and metadata
- **Usage System**: Argument validation and parsing based on script comments

## File-Specific Logic Audit

### 1. Help/Documentation Extraction (`src/parser.rs`)

| Function | Location | Responsibility | API | Coupling to CLI |
|----------|----------|----------------|-----|-----------------|
| `extract_initial_comment_block()` | `src/parser.rs:9-25` | Read initial comment block from script files | `fn(path: &Path) -> String` | **Low** - Pure file I/O |
| `extract_docs()` | `src/parser.rs:41-123` | Parse Summary, Usage, Options, Description from comments | `fn(path: &Path) -> Docs` | **Low** - Pure parsing logic |
| `provides_completions()` | `src/parser.rs:125-137` | Check if script provides completions | `fn(path: &Path) -> bool` | **Low** - Simple file scanning |

**Details:**
- Uses regex patterns to extract structured data from script comments
- Handles Summary, Usage, Options, and Description sections
- Pure file processing with no CLI dependencies
- Returns structured `Docs` with optional fields

### 2. Usage/Argument Processing (`src/usage.rs`)

| Function | Location | Responsibility | API | Coupling to CLI |
|----------|----------|----------------|-----|-----------------|
| `usage_parser()` | `src/usage.rs:67-99` | Parse usage comment syntax | Parser combinator | **Low** - Pure parsing logic |
| `option_parser()` | `src/usage.rs:48-65` | Parse option definitions | Parser combinator | **Low** - Pure parsing logic |
| `extract_usage()` | `src/usage.rs:242-291` | Extract usage from script file | `fn(config: &Config, path: &Path, cmd: &str) -> Usage` | **Medium** - Uses Config, returns clap Command |
| `Usage::parse_into_kv()` | `src/usage.rs:219-239` | Parse arguments into key-value pairs | `fn(&self, args: &Vec<String>) -> Result<String>` | **Low** - Pure data transformation |
| `Usage::get_next_option_name_for_completions()` | `src/usage.rs:184-217` | Get next option for completions | `fn(&self, args: &Vec<String>) -> Option<String>` | **Medium** - Uses clap internals |

**Details:**
- Complex parser combinators for usage syntax
- Handles positional args, flags, options, rest arguments
- Integrates with clap for command building
- Supports completion type specifications

### 3. Command Handling (`src/commands/file.rs`)

| Function | Location | Responsibility | API | Coupling to CLI |
|----------|----------|----------------|-----|-----------------|
| `FileCommand::new()` | `src/commands/file.rs:19-33` | Create file command from path | Constructor | **Medium** - Uses Config |
| `FileCommand::summary()` | `src/commands/file.rs:41-43` | Get command summary | `fn(&self) -> String` | **Low** - Delegates to Usage |
| `FileCommand::help()` | `src/commands/file.rs:51-55` | Generate help text | `fn(&self) -> Result<String>` | **Low** - Delegates to Usage |
| `FileCommand::completions()` | `src/commands/file.rs:62-122` | Handle completions | `fn(&self) -> Result<i32>` | **High** - Process execution, env vars |
| `FileCommand::invoke()` | `src/commands/file.rs:124-148` | Execute script file | `fn(&self) -> Result<i32>` | **High** - Process execution, env vars |

**Details:**
- Wraps Usage with file path and arguments
- Handles both old and new completion systems
- Sets CLI-specific environment variables
- Executes scripts with proper environment

### 4. Directory Processing (`src/commands/directory.rs`)

| Function | Location | Responsibility | API | Coupling to CLI |
|----------|----------|----------------|-----|-----------------|
| `DirectoryCommand::new()` | `src/commands/directory.rs:49-75` | Create directory command | Constructor | **Medium** - Uses Config, calls `parser::extract_docs()` |
| `DirectoryCommand::top_level()` | `src/commands/directory.rs:22-47` | Create top-level directory command | Constructor | **Medium** - Uses Config, calls `parser::extract_docs()` |

**Details:**
- Reads README files for directory descriptions
- Uses same parsing logic as file commands
- Integrates with clap command building

## Proposed Magic Module Structure

```
magic/
├── lib.rs              # Main module exports and re-exports
├── error.rs            # Magic-specific error types
├── help_parser.rs      # Help/documentation extraction
├── usage_parser.rs     # Usage comment parsing
├── option_parser.rs    # Option definition parsing  
├── metadata.rs         # Script metadata extraction
├── completion.rs       # Completion type handling
└── script.rs          # High-level script abstraction
```

## Extraction Plan

### Phase 1: Core File Processing (Low Coupling)
- **`magic/help_parser.rs`**: Extract `extract_initial_comment_block()`, `extract_docs()`, `provides_completions()`
- **`magic/metadata.rs`**: Extract `Docs` struct and related types
- **`magic/error.rs`**: Define magic-specific error types

### Phase 2: Usage Processing (Medium Coupling)
- **`magic/usage_parser.rs`**: Extract `usage_parser()`, `UsageLang`, `ArgSpec`, `ArgBase`
- **`magic/option_parser.rs`**: Extract `option_parser()`, `OptionSpec`, `CompletionType`
- **`magic/completion.rs`**: Extract completion-related logic

### Phase 3: Integration Layer
- **`magic/script.rs`**: High-level Script struct that combines all functionality
- **Update `sub`**: Modify to use magic module instead of internal parsing

## API Design

### Core Types
```rust
pub struct Script {
    path: PathBuf,
    metadata: Metadata,
    usage: Option<UsageSpec>,
    completion_info: CompletionInfo,
}

pub struct Metadata {
    summary: Option<String>,
    description: Option<String>,
    provides_completions: bool,
}

pub struct UsageSpec {
    arguments: Vec<ArgSpec>,
    rest: Option<String>,
}
```

### Main API Functions
```rust
pub fn parse_script(path: &Path) -> Result<Script, MagicError>;
pub fn extract_help(path: &Path) -> Result<Metadata, MagicError>;
pub fn extract_usage(path: &Path) -> Result<Option<UsageSpec>, MagicError>;
pub fn extract_options(path: &Path) -> Result<Vec<OptionSpec>, MagicError>;
```

## Boundary Analysis

### What Moves to Magic
- **Pure file parsing**: Comment extraction, regex parsing, file I/O
- **Language parsing**: Usage syntax, option definitions, completion types
- **Data structures**: Docs, ArgSpec, CompletionType, etc.
- **Validation**: Usage string validation, option validation

### What Stays in Sub
- **CLI integration**: clap Command building, CLI-specific args
- **Process execution**: Script invocation, environment variable setting
- **Command orchestration**: Command trait, directory traversal
- **Configuration**: CLI name, colors, paths, sub-specific settings

## Risk Assessment

### Low Risk
- File parsing functions are pure and well-tested
- Data structures are simple and isolated
- No breaking changes to external API

### Medium Risk
- Usage parsing is complex with many edge cases
- Integration with clap needs careful handling
- Completion system has two implementations

### High Risk
- Environment variable naming is CLI-specific
- Process execution is tightly coupled to sub
- Error handling spans multiple layers

## Next Steps

1. **Create magic module structure** with basic scaffolding
2. **Extract help parsing logic** (lowest risk, highest impact)
3. **Extract usage parsing logic** with tests
4. **Create Script abstraction** to unify functionality
5. **Update sub to use magic** with minimal changes
6. **Validate all functionality** works as before

## Success Criteria

- [ ] All existing tests pass
- [ ] No functionality regression
- [ ] Clean API boundary between magic and sub
- [ ] Reusable magic module for other projects
- [ ] Documentation for magic module API