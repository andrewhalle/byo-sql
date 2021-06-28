use super::{Identifier, List, Literal};

/// TODO quick description.
///
/// TODO long description.
#[derive(Debug)]
pub struct InsertQuery<'input> {
    table: Identifier<'input>,
    columns: Vec<Identifier<'input>>,
    values: Vec<Literal<'input>>,
}

use crate::parse::Rule;
use pest::iterators::Pair;

impl<'input> From<Pair<'input, Rule>> for InsertQuery<'input> {
    fn from(insert_query: Pair<'input, Rule>) -> Self {
        assert_eq!(insert_query.as_rule(), Rule::insert_query);

        let mut inner = insert_query.into_inner();
        let table = inner.next().unwrap().into();
        let columns: List<Identifier<'input>> = inner.next().unwrap().into();
        let values: List<Literal<'input>> = inner.next().unwrap().into();

        InsertQuery {
            table,
            columns: columns.0,
            values: values.0,
        }
    }
}
