# Changelog

## v2.3.0 - 20.06.2024

- Add support for literal commands in completions

## v2.2.0 - 20.06.2024

- Add new completions system. The old system is still supported so this isn't a
  breaking change. The new system has priority over the old system when both
  are present.

## v2.1.0 - 16.06.2024

- Rework `--validation` flag. It now needs to come after `--` and validates any
  command (not just the top level command)
- Improve arguments validation

## v2.0.0 - 23.05.2024

Release 2.0.0 is a major release that includes breaking changes.

- Rename `--bin` to `--executable`
- Replace `help` subcommand with `--help` flag
- Replace `completions` subcommand with `--completions` flag
- Replace `commands` subcommand with `--commands` flag
- Validate script arguments from Usage comment

Plus a major refactor of the codebase, new features and changes to the output.

## v1.1.0 - 03.05.2024

- Add flag `--extension` (`-e`) to `commands` subcommand to filter by file extension

## v1.0.0 - 31.03.2024

- Since 0.9.0 has been stable for so long, this is just a major version bump to
  start semantic versioning.

## v0.9.0 - 28.04.2023

- Package only M1 binary instead of Intel binary for OSX
