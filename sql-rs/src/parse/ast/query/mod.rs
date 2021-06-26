// TODO
// mod create_table_query;
// use create_table_query::*;
//
// mod insert_query;
// use insert_query::*;

mod select_query;
use select_query::*;

/// The root of the AST, representing a single query.
///
/// TODO long description.
#[derive(Debug)]
pub enum Query<'input> {
    SelectQuery(SelectQuery<'input>),
    //InsertQuery(InsertQuery<'input>),
    //CreateTableQuery(CreateTableQuery<'input>),
}

use crate::parse::Rule;
use pest::iterators::Pair;

impl<'input> From<Pair<'input, Rule>> for Query<'input> {
    fn from(query: Pair<'input, Rule>) -> Self {
        assert_eq!(query.as_rule(), Rule::query);

        let mut inner = query.into_inner();
        let query = inner.next().unwrap();
        match first.as_rule() {
            Rule::select_query => Query::SelectQuery(query.into()),
        }
    }
}

pub struct Queries<'input>(Vec<Query<'input>>);

impl<'input> From<Pair<'input, Rule>> for Queries<'input> {
    fn from(queries: Pair<'input, Rule>) -> Self {
        assert_eq!(queries.as_rule(), Rule::queries);

        Queries(queries.into_inner().map(<Query<'input as From<Pair<'input, Rule>>>>::from).collect())
    }
}
