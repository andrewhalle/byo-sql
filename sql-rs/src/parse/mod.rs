use pest::error::Error;
use pest::Parser;

mod parser;
use parser::{QueryParser, Rule};

pub mod ast;

/// The main entry function for the parser module. Parses a list of queries from a &str.
pub fn parse_queries(source: &str) -> Result<ast::Queries<'_>, Error<Rule>> {
    let mut parse = QueryParser::parse(Rule::queries, source)?;

    Ok(parse.next().unwrap().into())
}
