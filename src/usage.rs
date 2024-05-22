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
    Long(String, Option<String>),
}

#[derive(Debug, PartialEq)]
pub struct ArgSpec {
    base: ArgBase,
    required: bool,
    exclusive: bool,
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
    let value = filter(|c: &char| c.is_ascii_alphabetic() && c.is_uppercase()).repeated().at_least(1).map(|v| v.into_iter().collect::<String>());

    let positional = ident.padded().map(|s| ArgBase::Positional(s));
    let short = just("-").ignore_then(filter(|c: &char| c.is_alphabetic())).padded().map(|c| ArgBase::Short(c));
    let long = just("--").ignore_then(ident).then(just('=').ignore_then(value).or_not()).padded().map(|(k, v)| ArgBase::Long(k, v));

    let base_arg = positional.or(short).or(long);

    let optional = just('[').ignore_then(base_arg).then_ignore(just(']')).then(just('!').or_not().map(|e| e.is_some())).padded().map(|(s, e)| ArgSpec { base: s, required: false, exclusive: e });
    let required = base_arg.padded().map(|s| ArgSpec { base: s, required: true, exclusive: false });

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
        let input = "# Usage: {cmd} name -f --long [opt] [-o] [--longopt] [--value=VALUE] [--exclusive=EXCLUSIVE]!";
        let result = usage_parser().parse(input).unwrap();
        assert_eq!(result, UsageLang {
            arguments: vec![
                ArgSpec{ base: ArgBase::Positional("name".to_owned()), required: true, exclusive: false },
                ArgSpec{ base: ArgBase::Short('f'), required: true, exclusive: false },
                ArgSpec{ base: ArgBase::Long("long".to_owned(), None), required: true, exclusive: false },
                ArgSpec{ base: ArgBase::Positional("opt".to_owned()), required: false, exclusive: false },
                ArgSpec{ base: ArgBase::Short('o'), required: false, exclusive: false },
                ArgSpec{ base: ArgBase::Long("longopt".to_owned(), None), required: false, exclusive: false },
                ArgSpec{ base: ArgBase::Long("value".to_owned(), Some("VALUE".to_owned())), required: false, exclusive: false },
                ArgSpec{ base: ArgBase::Long("exclusive".to_owned(), Some("EXCLUSIVE".to_owned())), required: false, exclusive: true },
            ],
            rest: None,
        });
    }

    #[test]
    fn parse_with_rest() {
        let input = "# Usage: {cmd} name [opt] [rest]...";
        let result = usage_parser().parse(input).unwrap();
        assert_eq!(result, UsageLang {
            arguments: vec![
                ArgSpec{ base: ArgBase::Positional("name".to_owned()), required: true, exclusive: false },
                ArgSpec{ base: ArgBase::Positional("opt".to_owned()), required: false, exclusive: false },
            ],
            rest: Some("rest".to_owned()),
        });
    }
}

pub struct Usage {
    command: Command,
    error: Option<Error>,
}

impl Usage {
    pub fn new(command: Command, error: Option<Error>) -> Self {
        Self {
            command,
            error,
        }
    }

    pub fn generate(&self) -> String {
        self.command.clone().render_usage().ansi().to_string()
    }

    pub fn validate(&self) -> Result<()> {
        if let Some(error) = &self.error {
            return Err(error.clone());
        }

        Ok(())
    }

    pub fn summary(&self) -> String {
        self.command.get_about().map(|s| s.ansi().to_string()).unwrap_or_default()
    }

    pub fn help(&self) -> Result<String> {
        self.validate()?;

        Ok(self.command.clone().render_help().ansi().to_string())
    }

    pub fn parse_into_kv(&self, args: &Vec<String>) -> Result<String> {
        let clap_args = self.command.clone().get_matches_from(args);

        let mut args_parts = Vec::<String>::new();

        for arg in self.command.get_arguments() {
            if let Some(values) = clap_args.get_raw(arg.get_id().as_str()) {
                args_parts.push(arg.get_id().to_string());

                let mut value_parts = Vec::new();

                for v in values {
                    value_parts.push(v.to_str().ok_or(Error::InvalidUTF8)?.to_string());
                }

                args_parts.push(format!("\"{}\"", value_parts.join(" ")));
            }
        }

        Ok(args_parts.join(" "))
    }
}

pub fn extract_usage(config: &Config, path: &Path, cmd: &str) -> Usage {
    let docs = parser::extract_docs(&path);

    let mut command = config.base_command(cmd).no_binary_name(true);

    if let Some(summary) = docs.summary {
        command = command.about(summary);
    }

    if let Some(description) = docs.description {
        command = command.after_help(description);
    }

    let mut error = None;

    if let Some(line) = docs.usage {
        match usage_parser().parse(line) {
            Ok(usage_lang) => {
                command = apply_arguments(command, usage_lang);
            },
            Err(e) => error = Some(Error::InvalidUsageString(e)),
        }
    } else {
        command = command.arg(Arg::new("args").trailing_var_arg(true).num_args(..).allow_hyphen_values(true));
    }

    return Usage::new(command, error);
}

fn apply_arguments(mut command: Command, usage_lang: UsageLang) -> Command {
    for arg in usage_lang.arguments {
        let mut clap_arg = match arg.base {
            ArgBase::Positional(ref name) => {
                Arg::new(name).required(true)
            }
            ArgBase::Short(character) => {
                Arg::new(character.to_string()).short(character).num_args(0).required(true)
            }
            ArgBase::Long(ref name, value) => {
                let mut arg = Arg::new(name).long(name).required(true);
                if let Some(value) = value {
                    arg = arg.num_args(1).value_name(value);
                } else {
                    arg = arg.num_args(0);
                }
                arg
            }
        };

        clap_arg = clap_arg.exclusive(arg.exclusive);
        clap_arg = clap_arg.required(arg.required);

        command = command.arg(clap_arg);
    }

    if let Some(rest) = usage_lang.rest {
        command = command.arg(Arg::new(rest).trailing_var_arg(true).num_args(..).allow_hyphen_values(true));
    }

    command
}
