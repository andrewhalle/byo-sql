use pest::error::Error;
use pest::Parser;

mod parser;
// TODO remove pub
pub use parser::{QueryParser, Rule};

mod ast;
pub use ast::*;

pub fn parse_queries(source: &str) -> Result<Queries<'_>, Error<Rule>> {
    let mut parse = QueryParser::parse(Rule::queries, source)?;

    Ok(parse.next().unwrap().into())
}
