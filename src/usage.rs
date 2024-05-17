extern crate regex;
extern crate clap;

use chumsky::prelude::*;
use clap::{Command, Arg};

use std::path::Path;

use crate::parser;
use crate::error::{Error, Result};
use crate::config::Config;

#[derive(Debug, PartialEq)]
pub enum ArgBase {
    Positional(String),
    Short(char),
    Long(String),
}

#[derive(Debug, PartialEq)]
pub enum ArgSpec {
    Required(ArgBase),
    Optional(ArgBase),
}

#[derive(Debug, PartialEq)]
struct UsageLang {
    arguments: Vec<ArgSpec>,
    rest: Option<String>,
}

fn usage_parser() -> impl Parser<char, UsageLang, Error = Simple<char>> {
    let prefix = just("# Usage:").padded();

    let cmd_token = just("{cmd}").padded();

    let ident = text::ident().map(|s: String| s);

    let positional = ident.padded().map(|s| ArgBase::Positional(s));
    let short = just("-").ignore_then(filter(|c: &char| c.is_alphabetic())).padded().map(|c| ArgBase::Short(c));
    let long = just("--").ignore_then(ident).padded().map(|s| ArgBase::Long(s));

    let base_arg = positional.or(short).or(long);

    let optional = just('[').ignore_then(base_arg).then_ignore(just(']')).padded().map(|s| ArgSpec::Optional(s));
    let required = base_arg.padded().map(|s| ArgSpec::Required(s));

    let argument = optional.or(required).then_ignore(none_of(".").ignored().or(end()).rewind());

    let rest = just('[').ignore_then(ident).then_ignore(just("]...")).padded();

    prefix.ignore_then(cmd_token).ignore_then(argument.repeated()).then(rest.or_not()).then_ignore(end()).map(|(args, rest)| {
        UsageLang {
            arguments: args,
            rest,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_without_rest() {
        let input = "# Usage: {cmd} name -f --long [opt] [-o] [--longopt]";
        let result = usage_parser().parse(input).unwrap();
        assert_eq!(result, UsageLang {
            arguments: vec![
                ArgSpec::Required(ArgBase::Positional("name".to_owned())),
                ArgSpec::Required(ArgBase::Short('f')),
                ArgSpec::Required(ArgBase::Long("long".to_owned())),
                ArgSpec::Optional(ArgBase::Positional("opt".to_owned())),
                ArgSpec::Optional(ArgBase::Short('o')),
                ArgSpec::Optional(ArgBase::Long("longopt".to_owned())),
            ],
            rest: None,
        });
    }

    #[test]
    fn parse_with_rest() {
        let input = "# Usage: {cmd} name -f --long [opt] [-o] [--longopt] [rest]...";
        let result = usage_parser().parse(input).unwrap();
        assert_eq!(result, UsageLang {
            arguments: vec![
                ArgSpec::Required(ArgBase::Positional("name".to_owned())),
                ArgSpec::Required(ArgBase::Short('f')),
                ArgSpec::Required(ArgBase::Long("long".to_owned())),
                ArgSpec::Optional(ArgBase::Positional("opt".to_owned())),
                ArgSpec::Optional(ArgBase::Short('o')),
                ArgSpec::Optional(ArgBase::Long("longopt".to_owned())),
            ],
            rest: Some("rest".to_owned()),
        });
    }
}

pub struct Usage {
    command: Command,
}

impl Usage {
    pub fn from_command(command: Command) -> Self {
        Self {
            command,
        }
    }

    pub fn generate(&self) -> String {
        self.command.clone().render_usage().ansi().to_string()
    }

    pub fn command(&self) -> Command {
        self.command.clone()
    }
}

pub fn extract_usage(config: &Config, path: &Path, cmd: &str) -> Result<Usage> {
    let docs = parser::extract_docs(&path);

    let mut command = config.base_command(cmd).no_binary_name(true);

    if let Some(summary) = docs.summary {
        command = command.about(summary);
    }

    if let Some(description) = docs.description {
        command = command.after_help(description);
    }

    if let Some(line) = docs.usage {
        match usage_parser().parse(line) {
            Ok(usage_lang) => {
                command = apply_arguments(command, usage_lang);
            },
            Err(e) => return Err(Error::InvalidUsageString(e)),
        }
    } else {
        command = command.arg(Arg::new("args").trailing_var_arg(true).num_args(..).allow_hyphen_values(true));
    }

    return Ok(Usage::from_command(command));
}

fn apply_arguments(mut command: Command, usage_lang: UsageLang) -> Command {
    for arg in usage_lang.arguments {
        match arg {
            ArgSpec::Required(base) => {
                match base {
                    ArgBase::Positional(ref name) => {
                        command = command.arg(Arg::new(name).required(true));
                    }
                    ArgBase::Short(character) => {
                        command = command.arg(Arg::new(character.to_string()).short(character).num_args(0).required(true));
                    }
                    ArgBase::Long(ref name) => {
                        command = command.arg(Arg::new(name).long(name).num_args(0).required(true));
                    }
                }
            },
            ArgSpec::Optional(base) => {
                match base {
                    ArgBase::Positional(ref name) => {
                        command = command.arg(Arg::new(name).required(false));
                    }
                    ArgBase::Short(character) => {
                        command = command.arg(Arg::new(character.to_string()).short(character).num_args(0).required(false));
                    }
                    ArgBase::Long(ref name) => {
                        command = command.arg(Arg::new(name).long(name).num_args(0).required(false));
                    }
                }
            },
        }
    }

    if let Some(rest) = usage_lang.rest {
        command = command.arg(Arg::new(rest).trailing_var_arg(true).num_args(..).allow_hyphen_values(true));
    }

    command
}
