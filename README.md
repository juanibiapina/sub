# Sub

Organize groups of scripts into documented CLIs with subcommands.

## Overview

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

```
#!/usr/bin/env bash

sub --name awesomecli --executable "${BASH_SOURCE[0]}" --relative ".." -- "$@"
```

The `--name` argument tells `sub` the name of the CLI. This is used when
printing help information.

The `--executable` argument tells `sub` where the CLI entry point is located.
Usually this will just be `${BASH_SOURCE[0]}`. The `--relative` argument tells
`sub` how to find the root of the CLI starting from the CLI entry point.
These two are separate arguments for cross platform compatibility. `sub` will
canonalize the bin path before merging with the relative path and then canonalize
again.

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

To get help for a command, use the built in `help` command:

```
$ awesomecli help <commandname>
```

In order to display help information, `sub` looks for special comments in each
file. An example documented command:

```sh
#!/usr/bin/env bash
#
# Summary: One line summary of the command
#
# Usage: {cmd} <required-arg>
#        {cmd} [--option] <required-arg>
#
#  --option Activates an option
#
# Extended description of what the command does.
#
# The extended description can span multiple lines.
```

If the command is a directory, `sub` looks for documentation in a `README` file
inside that directory.

## Sharing code

When invoking subcommands, `sub` sets an environment variable called
`_CLINAME_ROOT` (where `CLINAME` is the name of your CLI. This variable holds
the canonicalized path to the root of your CLI. It can be used for instance for
sourcing shared scripts.

## Caching

When invoking subcommands, `sub` sets an environment variable called
`_CLINAME_CACHE` (where `CLINAME` is the name of your CLI. This variable points
to an XDG compliant cache directory that can be used for storing temporary files.

## Inspiration

- [sub from basecamp](https://github.com/basecamp/sub)
- [sd](https://github.com/cv/sd)
