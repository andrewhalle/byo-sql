// TODO make private once Query is moved into crate::parse::ast.
#[derive(Parser)]
#[grammar = "parse/query.pest"]
pub struct QueryParser;
