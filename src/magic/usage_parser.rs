extern crate chumsky;

use chumsky::prelude::*;
use super::error::{MagicError, Result};

#[derive(Debug, PartialEq, Clone)]
pub enum ArgBase {
    Positional(String),
    Short(char),
    Long(String, Option<String>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ArgSpec {
    pub base: ArgBase,
    pub required: bool,
    pub exclusive: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct UsageLang {
    pub arguments: Vec<ArgSpec>,
    pub rest: Option<String>,
}

pub fn usage_parser() -> impl Parser<char, UsageLang, Error = Simple<char>> {
    let prefix = just("# Usage:").padded();

    let cmd_token = just("{cmd}").padded();

    let ident = filter(|c: &char| c.is_ascii_alphabetic())
        .chain(filter(|c: &char| c.is_ascii_alphanumeric() || *c == '_' || *c == '-').repeated())
        .collect();
    let value = filter(|c: &char| c.is_ascii_alphabetic() && c.is_uppercase()).repeated().at_least(1).map(|v| v.into_iter().collect::<String>());

    let short = just("-").ignore_then(filter(|c: &char| c.is_alphabetic())).padded().map(|c| ArgBase::Short(c));
    let long = just("--").ignore_then(ident).then(just('=').ignore_then(value).or_not()).padded().map(|(k, v)| ArgBase::Long(k, v));

    let optional_positional = ident.padded().map(|s| ArgBase::Positional(s));
    let required_positional = just('<').ignore_then(ident).then_ignore(just('>')).padded().map(|s| ArgBase::Positional(s));

    let in_optional = short.or(long).or(optional_positional);
    let in_required = short.or(long).or(required_positional);

    let optional = just('[').ignore_then(in_optional).then_ignore(just(']')).then(just('!').or_not().map(|e| e.is_some())).padded().map(|(s, e)| ArgSpec { base: s, required: false, exclusive: e });
    let required = in_required.padded().map(|s| ArgSpec { base: s, required: true, exclusive: false });

    let argument = optional.or(required).then_ignore(none_of(".").ignored().or(end()).rewind());

    let rest = just('[').ignore_then(ident).then_ignore(just("]...")).padded();

    prefix.ignore_then(cmd_token).ignore_then(argument.repeated()).then(rest.or_not()).then_ignore(end()).map(|(args, rest)| {
        UsageLang {
            arguments: args,
            rest,
        }
    })
}

pub fn parse_usage_line(line: &str) -> Result<UsageLang> {
    usage_parser().parse(line)
        .map_err(|e| MagicError::ParseError(format!("Usage parse error: {:?}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_without_rest() {
        let input = "# Usage: {cmd} <name> <m2-_m> -f --long [opt] [-o] [--longopt] [--value=VALUE] [--exclusive=EXCLUSIVE]!";
        let result = usage_parser().parse(input).unwrap();
        assert_eq!(result, UsageLang {
            arguments: vec![
                ArgSpec{ base: ArgBase::Positional("name".to_owned()), required: true, exclusive: false },
                ArgSpec{ base: ArgBase::Positional("m2-_m".to_owned()), required: true, exclusive: false },
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
        let input = "# Usage: {cmd} <name> [opt] [rest]...";
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