# Sub

Organize groups of scripts into documented CLIs with subcommands.

`sub` is a tool designed to help organize groups of scripts into a command-line
interface (CLI) with subcommands. It allows the dynamic creation of a CLI from
a directory (and subdirectories) of scripts.

Use the Github table of contents on the top right of this README to navigate
the documentation.

## Key features

- **Display help:** Display usage and documentation for scripts.
- **Validate arguments:** Validate arguments to scripts based on documentation.
- **Parse arguments:** Automatically parse arguments to scripts so `getopts` is not needed.
- **Nested subcommands:** Supports nested directories for hierarchical command structures.
- **Aliases:** Supports aliases for subcommands.
- **Completions:** Supports auto completion of subcommands.
- **Cross-platform:** Works on Linux and macOS.

## Installation

### Homebrew

```sh
brew install juanibiapina/tap/sub
```

### Nix with Flakes

Add sub to your flake inputs:

```nix
{
  inputs = {
    sub = {
      url = "github:juanibiapina/sub";
      inputs.nixpkgs.follows = "nixpkgs"; # Optional
    };
  };

  # ...
}
```

Then add it to your packages:

```nix
{
  environment.systemPackages = with pkgs; [
    inputs.sub.packages."${pkgs.system}".sub
    # ...
  ];
}
```

## Usage

`sub` is meant to be used as the entry point for a CLI. Given the following
directory structure:

```
.
├── bin
│   └── awesomecli
└── libexec
    ├── list
    ├── new
    ├── open
    └── nested
        ├── README
        └── command
```

The entry point in `bin/awesomecli` can then be:

```bash
#!/usr/bin/env bash

sub --name awesomecli --executable "${BASH_SOURCE[0]}" --relative ".." -- "$@"
```

The `--name` argument tells `sub` the name of the CLI. This is used when
printing help information.

The `--executable` argument tells `sub` where the CLI entry point is located.
Usually this will just be `${BASH_SOURCE[0]}`. The `--relative` argument tells
`sub` how to find the root of the CLI starting from the CLI entry point.

After the root directory is determined, `sub` picks up any executable files in
a `libexec` directory inside root to use as subcommands. Directories create
nested subcommands. Arguments for the subcommands themselves go after the `--`.

With this setup, invoking `awesomecli` will display all available subcommands
with associated help information. To invoke a subcommand, use:

```
$ awesomecli <subcommandname> <args>
```

Or to invoke a nested subcommand:

```
$ awesomecli nested command
```

## Documenting commands

To get help for a command, use the built in `--help` flag:

```sh
awesomecli --help <commandname>
```
or
```sh
awesomecli <commandname> --help
```

In order to display help information, `sub` looks for special comments in the
corresponding script. An example documented command:

```sh
#!/usr/bin/env bash
#
# Summary: One line summary of the command
#
# Usage: {cmd} <positional-arg>
#
# Extended description of what the command does.
#
# The extended description can span multiple lines.

set -e

echo "Hello $1"
```

If the command is a directory, `sub` looks for documentation in a `README` file
inside that directory.

## Sharing code between scripts

When invoking subcommands, `sub` sets an environment variable called
`_CLINAME_ROOT` (where `CLINAME` is the name of your CLI. This variable holds
the canonicalized path to the root of your CLI. It can be used for instance for
sourcing shared scripts from a `lib` directory next to `libexec`:

```sh
source "$_CLINAME_ROOT/lib/shared.sh"
```

## Caching

When invoking subcommands, `sub` sets an environment variable called
`_CLINAME_CACHE` (where `CLINAME` is the name of your CLI. This variable points
to an XDG compliant cache directory that can be used for storing temporary files.

## Migrating to Sub 2.x

### --bin was renamed to --executable

The `--bin` argument was renamed to `--executable` to better reflect its purpose.

### Usage comments

Sub 2.x introduces automatic validation and parsing of command line arguments
based on special Usage comments in scripts. If you previously used arbitrary
Usage comments in sub 1.x for the purpose of documenting, you can run `sub`
with the `--validate` flag to check if your scripts are compatible with the new
version.

### Help, commands and completions

If you used the `help`, `commands` or `completions` subcommands, they are now
`--help`, `--commands` and `--completions` flags respectively.

## Inspiration

- [sub from basecamp](https://github.com/basecamp/sub)
- [sd](https://github.com/cv/sd)
