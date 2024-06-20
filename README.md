# Sub

Scripts with superpowers.

`sub` is a tool for organizing scripts into a unified command-line interface.
It allows the dynamic creation of a CLI from a directory (and subdirectories)
of scripts with support for documentation, argument validation, and
completions.

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

## Demo

[![asciicast](https://asciinema.org/a/664235.svg)](https://asciinema.org/a/664235)

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

This section explains how to set up a CLI called `hat` using `sub`.

### Examples

For a simple example, check out a [hello
world](https://github.com/juanibiapina/sub/tree/master/examples/hello) project.

For a complete example with lots of features, check out the
[complete](https://github.com/juanibiapina/sub/tree/master/examples/complete)
example project.

### As an alias

The quickest way to get started with `sub` is to define an alias for your CLI
in your shell:

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

```sh
#!/usr/bin/env bash

sub --name hat --executable "${BASH_SOURCE[0]}" --relative ".." -- "$@"
```

The `--executable` argument tells `sub` where the CLI entry point is located.
This will almost always be `${BASH_SOURCE[0]}`. The `--relative` argument tells
`sub` how to find the root of the CLI starting from the CLI entry point. In the
line above, just replace `hat` with the name of your CLI.

## Usage

Once you have set up your CLI (we called it `hat`), you can get help by running:

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

`sub` looks for special comments in a comment block in the beginning of the
file. The special comments are:

- `Summary:` A short description of the script.
- `Usage:` A description of the arguments the script accepts. Note that the
  Usage comment, when present, has specific syntactic rules and is used to
  parse command line arguments. See [Validating arguments](#validating-arguments)
  and [Parsing arguments](#parsing-arguments) for more information.
- `Options:` A description of the options the script accepts. This is used to
  display help information and generate completions. See
  [Completions](#completions) for more details.
- Extended documentation: Any other comment lines in this initial block will be
  considered part of the extended documentation.

## Validating arguments

`sub` automatically validates arguments to scripts based on the `Usage`
comment when it is present. The syntax for the `Usage` comment is:

```
# Usage: {cmd} <positional> [optional] [-u] [--long] [--value=VALUE] [--exclusive]! [rest]...
```

- `{cmd}`: This special token represents the name of the command and is always required.
- `<positional>`: A required positional argument.
- `[optional]`: An optional positional argument.
- `[-u]`: An optional short flag.
- `[--long]`: An optional long flag.
- `[--value=VALUE]`: An optional long flag that takes a value.
- `[--exclusive]!`: An optional long flag that cannot be used with other flags.
- `[rest]...`: A rest argument that consumes all remaining arguments.

Short and long flags can also be made required by omitting the brackets.

When invoking a script with invalid arguments, `sub` will display an error. For
example, invoking the `hello` script from the previous section with invalid
arguments:

```sh
$ hat hello
```

```
error: the following required arguments were not provided:
  <name>

Usage: hat hello --spanish <name>

For more information, try '--help'.
```

## Parsing arguments

When arguments to a script are valid, `sub` sets an environment variable called
`_HAT_ARGS` (where `HAT` is the capitalized name of your CLI). This variable
holds the parsed arguments as a list of key value pairs. The value of this
variable is a string that can be evaluated to an associative array in bash
scripts:

```sh
declare -A args="($_HAT_ARGS)"
```

Which can then be used to access argument values:

```sh
echo "${args[positional]}"

if [[ "${args[long]}" == "true" ]]; then
  # ...
fi
```

## Completions

`sub` automatically provides completions for subcommand names.

To enable completions for positional arguments in the `Usage` comment, add an
`Options:` comment with a list of arguments. An option must have the format:
`name (completion_type): description`. Completion type is optional. Currently,
the only supported completion type is `script`, which allows for dynamic
completions like the following example:

```sh
# Usage: {cmd} <name>
# Options:
#   name (script): A name

# check if we're being requested completions
if [[ "$_HAT_COMPLETE" == "true" ]]; then
  if [[ "$_HAT_COMPLETE_ARG" == "name" ]]; then
    echo "Alice"
    echo "Bob"
    echo "Charlie"
    # note that you can run any command here to generate completions
  fi

  # make sure to exit when generating completions to prevent the script from running
  exit 0
fi

# read arguments
declare -A args="($_HAT_ARGS)"

# say hello
echo "Hello, ${args[name]}!"
```


## Nested subcommands

`sub` supports nested directories for hierarchical command structures. For
example, given the following directory structure:

```
.
└── libexec
    └── nested
        ├── README
        └── user-script2
```

`user-script2` can be invoked with:

```sh
$ hat nested user-script2
```

Directories can be nested arbitrarily deep.

A `README` file can be placed in a directory to provide a description of the
subcommands in that directory. The `README` file should be formatted like a
script, with a special comment block at the beginning:

```sh
# Summary: A collection of user scripts
#
# This directory contains scripts that do magic.
# This help can be as long as you want.
# The Usage comment is ignored in README files.
```

## Aliases

To define an alias, simply create a symlink. For example, in the `libexec`
directory:

```sh
ln -s user-script1 us1
```

Aliases can also point to scripts in subdirectories:
```sh
ln -s nested/user-script2 us2
```

The full power of symlinks can be used to create complex command structures.

## Sharing code between scripts

When invoking subcommands, `sub` sets an environment variable called
`_HAT_ROOT` (where `HAT` is the capitalized name of your CLI. This variable
holds the path to the root of your CLI. It can be used, for instance, for
sourcing shared scripts from a `lib` directory next to `libexec`:

```sh
source "$_CLINAME_ROOT/lib/shared.sh"
```

## Caching

When invoking subcommands, `sub` sets an environment variable called
`_HAT_CACHE` (where `HAT` is the capitalized name of your CLI. This variable
points to an XDG compliant cache directory that can be used for storing
temporary files shared between subcommands.

## Migrating to Sub 2.x

### change --bin to --executable

The `--bin` argument was renamed to `--executable` to better reflect its purpose.

### Usage comments

Sub 2.x introduces automatic validation and parsing of command line arguments
based on special Usage comments in scripts. If you previously used arbitrary
Usage comments in sub 1.x for the purpose of documenting, you can run `sub`
with the `--validate` flag to check if your scripts are compatible with the new
version.

Example:
```
$ sub --name hat --absolute /path/to/cli/root -- --validate
```

### Help, commands and completions

If you used the `help`, `commands` or `completions` subcommands, they are now
`--help`, `--commands` and `--completions` flags respectively.

## Inspiration

- [sub from basecamp](https://github.com/basecamp/sub)
- [sd](https://github.com/cv/sd)
