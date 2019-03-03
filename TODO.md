# Bugs

- remove empty OPTIONS section

# Features

- add support for nested commands as directories
- extract documentation for file commands
- extract documentation for directory commands
- allow command specific completions
- generate completions
- provide access to the root directory
- accept arguments on help subcommand

# Tests

- rewrite tests using rust

# Refactor

- create a command struct and derive the clap app from it
