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

## Setup

### As an alias

The quickest way to get started with `sub` is to define an alias for your CLI
(let's call it `hat`) in your shell:

```sh
alias hat='sub --name hat --absolute /path/to/cli/root --'
```

Where `/path/to/cli/root` contains a `libexec` directory with executable
scripts, for example:

```
.
└── libexec
    ├── user-script1
    ├── user-script2
    └── user-script3
```

### As an executable

A more reliable way is to use an executable script as the entry point. Given
the following directory structure:

```
.
├── bin
│   └── hat
└── libexec
    ├── user-script1
    ├── user-script2
    └── user-script3
```

The entry point in `bin/hat` is then:

```bash
#!/usr/bin/env bash

sub --name hat --executable "${BASH_SOURCE[0]}" --relative ".." -- "$@"
```

The `--executable` argument tells `sub` where the CLI entry point is located.
This will almost always be `${BASH_SOURCE[0]}`. The `--relative` argument tells
`sub` how to find the root of the CLI starting from the CLI entry point. In the
line above, just replace `hat` with the name of your CLI.

## Usage

Once you have set up your CLI, you can get help by running:

```sh
$ hat --help
```
```
Usage: hat [OPTIONS] [commands_with_args]...

Arguments:
  [commands_with_args]...

Options:
      --usage                  Print usage
  -h, --help                   Print help
      --completions            Print completions
      --commands               Print subcommands
      --extension <extension>  Filter subcommands by extension

Available subcommands:
    user-script1
    user-script2
    user-script3
```

To invoke a subcommand, use:

```
$ hat user-script1
```

To get help for a command, use the built in `--help` flag:

```sh
hat --help <commandname>
```
or
```sh
hat <commandname> --help
```

## Documenting commands

In order to display help information, `sub` looks for special comments in the
corresponding script. A fully documented `hello` script could look like this:

```sh
#!/usr/bin/env bash
#
# Summary: Say hello
#
# Usage: {cmd} <name> [--spanish]
#
# Say hello to a user by name.
#
# With the --spanish flag, the greeting will be in Spanish.

set -e

declare -A args="($_HAT_ARGS)"

if [[ "${args[spanish]}" == "true" ]]; then
  echo "¡Hola, ${args[name]}!"
else
  echo "Hello, ${args[name]}!"
fi
```

## Sharing code between scripts

When invoking subcommands, `sub` sets an environment variable called
`_HAT_ROOT` (where `HAT` is the capitalized name of your CLI. This variable
holds the canonicalized path to the root of your CLI. It can be used for
instance for sourcing shared scripts from a `lib` directory next to `libexec`:

```sh
source "$_CLINAME_ROOT/lib/shared.sh"
```

## Caching

When invoking subcommands, `sub` sets an environment variable called
`_HAT_CACHE` (where `HAT` is the capitalized name of your CLI. This variable
points to an XDG compliant cache directory that can be used for storing
temporary files.

## Migrating to Sub 2.x

### change --bin was to --executable

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
