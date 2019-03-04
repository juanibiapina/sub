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

sub --name awesomecli --root "$0"/../.. -- "$@"
```

The `--name` argument tells `sub` the name of the CLI. This is used when
printing help information. The `--root` argument tells `sub` where to look for
subcommands. `sub` picks up any files in a `libexec` directory inside root to
use as subcommands. Note that root can be a relative path. `sub` will
canonicalize this path later. Arguments for the subcommands themselves go after
the `--`.

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
