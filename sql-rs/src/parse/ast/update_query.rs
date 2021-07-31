use super::{Assignment, Expression, Identifier, List};

/// TODO quick description.
///
/// TODO long description.
#[derive(Debug)]
pub struct UpdateQuery<'input> {
    pub table: Identifier<'input>,
    pub assignments: Vec<Assignment<'input>>,
    pub filter: Expression<'input>,
}

use crate::parse::Rule;
use pest::iterators::Pair;

impl<'input> From<Pair<'input, Rule>> for UpdateQuery<'input> {
    fn from(update_query: Pair<'input, Rule>) -> Self {
        assert_eq!(update_query.as_rule(), Rule::update_query);

        let mut inner = update_query.into_inner();

        let table = inner.next().unwrap().into();
        let assignments: List<Assignment<'input>> = inner.next().unwrap().into();
        let mut where_clause = inner.next().unwrap().into_inner();
        let filter = where_clause.next().unwrap().into();

        UpdateQuery {
            table,
            assignments: assignments.0,
            filter,
        }
    }
}
