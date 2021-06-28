use super::{Join, TableIdentifier};

/// TODO quick description.
///
/// TODO long description.
#[derive(Debug)]
pub struct TableExpression<'input> {
    root_table: TableIdentifier<'input>,
    joins: Vec<Join<'input>>,
}

use crate::parse::Rule;
use pest::iterators::Pair;

impl<'input> From<Pair<'input, Rule>> for TableExpression<'input> {
    fn from(table_expression: Pair<'input, Rule>) -> Self {
        assert_eq!(table_expression.as_rule(), Rule::table_expression);

        let mut inner = table_expression.into_inner();
        let root_table = inner.next().unwrap().into();
        let joins = inner.map(From::from).collect();

        TableExpression { root_table, joins }
    }
}
