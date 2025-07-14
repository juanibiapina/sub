extern crate chumsky;

use chumsky::prelude::*;
use super::error::{MagicError, Result};
use super::completion::CompletionType;

#[derive(Debug, PartialEq, Clone)]
pub struct OptionSpec {
    pub name: String,
    pub completion_type: Option<CompletionType>,
    pub description: Option<String>,
}

pub fn option_parser() -> impl Parser<char, OptionSpec, Error = Simple<char>> {
    let ident = filter(|c: &char| c.is_ascii_alphabetic())
        .chain(filter(|c: &char| c.is_ascii_alphanumeric() || *c == '_' || *c == '-').repeated())
        .collect();

    let completion_type_script = just("script").map(|_| CompletionType::Script);
    let completion_type_literal_command = just('`').ignore_then(take_until(just('`')).padded()).map(|(s, _)| CompletionType::LiteralCommand(s.into_iter().collect()));

    let completion_type = completion_type_script.or(completion_type_literal_command);

    let description = take_until(end()).padded().map(|(s, _)| s.into_iter().collect());

    ident.padded().then(completion_type.delimited_by(just('('), just(')')).or_not()).then_ignore(just(':')).then(description.padded()).map(|((name, completion_type), description)| OptionSpec {
        name,
        completion_type,
        description: Some(description),
    })
}

pub fn parse_option_line(line: &str) -> Result<OptionSpec> {
    option_parser().parse(line)
        .map_err(|e| MagicError::ParseError(format!("Option parse error: {:?}", e)))
}