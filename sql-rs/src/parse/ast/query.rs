use super::{CreateTableQuery, InsertQuery, SelectQuery, UpdateQuery};

/// The root of the AST, representing a single query.
///
/// TODO long description.
#[derive(Debug)]
pub enum Query<'input> {
    SelectQuery(SelectQuery<'input>),
    InsertQuery(InsertQuery<'input>),
    CreateTableQuery(CreateTableQuery<'input>),
    UpdateQuery(UpdateQuery<'input>),
}

use crate::parse::Rule;
use pest::iterators::Pair;

impl<'input> From<Pair<'input, Rule>> for Query<'input> {
    fn from(query: Pair<'input, Rule>) -> Self {
        assert_eq!(query.as_rule(), Rule::query);

        let mut inner = query.into_inner();
        let query = inner.next().unwrap();
        match query.as_rule() {
            Rule::select_query => Query::SelectQuery(query.into()),
            Rule::insert_query => Query::InsertQuery(query.into()),
            Rule::create_table_query => Query::CreateTableQuery(query.into()),
            Rule::update_query => Query::UpdateQuery(query.into()),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct Queries<'input>(pub Vec<Query<'input>>);

impl<'input> From<Pair<'input, Rule>> for Queries<'input> {
    fn from(queries: Pair<'input, Rule>) -> Self {
        assert_eq!(queries.as_rule(), Rule::queries);

        Queries(queries.into_inner().map(From::from).collect())
    }
}
