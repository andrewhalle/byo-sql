use super::{Expression, List, OrderBy, TableExpression};

/// TODO quick description.
///
/// TODO long description.
#[derive(Debug)]
pub struct SelectQuery<'input> {
    select_list: Vec<Expression<'input>>,
    table: TableExpression<'input>,
    filter: Option<Expression<'input>>,
    sort: Option<OrderBy<'input>>,
    limit: Option<Expression<'input>>,
}

use crate::parse::Rule;
use pest::iterators::Pair;

impl<'input> From<Pair<'input, Rule>> for SelectQuery<'input> {
    fn from(select_query: Pair<'input, Rule>) -> Self {
        assert_eq!(select_query.as_rule(), Rule::select_query);

        let mut inner = select_query.into_inner();

        let select_list: List<Expression<'input>> = inner.next().unwrap().into();
        let table = inner.next().unwrap().into();
        let mut filter = None;
        let mut sort = None;
        let mut limit = None;

        loop {
            let pair = inner.next();
            match pair {
                Some(pair) => match pair.as_rule() {
                    Rule::where_clause => filter = Some(pair.into_inner().next().unwrap().into()),
                    Rule::order_by_clause => sort = Some(pair.into()),
                    Rule::limit_clause => limit = Some(pair.into_inner().next().unwrap().into()),
                    _ => unreachable!(),
                },
                None => break,
            }
        }

        SelectQuery {
            select_list: select_list.0,
            table,
            filter,
            sort,
            limit,
        }
    }
}
