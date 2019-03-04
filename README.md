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
    └── open
```

The entry point in `bin/awesomecli` can then be:

```
#!/usr/bin/env bash

sub --name awesomecli --bin "${BASH_SOURCE[0]}" --relative ".." -- "$@"
```

The `--name` argument tells `sub` the name of the CLI. This is used when
printing help information.

The `--bin` argument tells `sub` where the binary entry point is located.
Usually this will just be `${BASH_SOURCE[0]}`. The `--relative` argument tells
`sub` how to find the root of the CLI starting from the binary entry point.
These two are separate arguments for cross platform compatibility. `sub` will
canonalize the bin path before merging with the relative path and then canonalize
again.

After the root directory is determined, `sub` picks up any files in a `libexec`
directory inside root to use as subcommands. Arguments for the subcommands
themselves go after the `--`.

With this setup, invoking `awesomecli` will display all available subcommands
with associated help information. To invoke a subcommand, use:

```
$ awesomecli <commandname> <args>
```

## Documenting commands

To get help for a command, use the built in `help` command:

```
$ awesomecli help <commandname>
```

In order to display help information, `sub` looks for special comments in the
beginning of each file. An example documented command:

```sh
#!/usr/bin/env bash
# Summary: One line summary of the command
# Help: Longer description of what the command does.
#
# The help section can span multiple lines.
```

## Sharing code

When invoking subcommands, `sub` sets an environment variable called
`_CLINAME_ROOT` (where `CLINAME` is the name of your CLI. This variable holds
the canonicalized path to the root of your CLI. It can be used for instance for
sourcing shared scripts.

## Inspiration

- [sub from basecamp](https://github.com/basecamp/sub)
- [sd](https://github.com/cv/sd)