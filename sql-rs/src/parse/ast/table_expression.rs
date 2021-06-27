/// TODO quick description.
///
/// TODO long description.
#[derive(Debug)]
// pub struct TableExpression<'input>;
pub struct TableExpression;

use crate::parse::Rule;
use pest::iterators::Pair;

impl From<Pair<'_, Rule>> for TableExpression {
    fn from(table_expression: Pair<'_, Rule>) -> Self {
        assert_eq!(table_expression.as_rule(), Rule::table_expression);

        // let mut inner = <name>.into_inner();
        // impl

        // construct instance
        TableExpression
    }
}
