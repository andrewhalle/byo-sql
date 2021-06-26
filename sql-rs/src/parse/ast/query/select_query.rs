/// TODO quick description.
///
/// TODO long description.
#[derive(Debug)]
struct SelectQuery<'input> {
    select_list: Vec<Expression<'input>>,
    table: TableSelection<'input>,
    filter: Option<Expression<'input>>,
    sort: Option<SortClause<'input>>,
    limit: Option<Literal<'input>>,
}

use crate::parse::Rule;
use pest::iterators::Pair;

impl<'input> From<Pair<'input, Rule>> for SelectQuery<'input> {
    fn from(select_query: Pair<'input, Rule>) -> Self {
        assert_eq!(select_query.as_rule(), Rule::select_query);

        let mut inner = select_query.into_inner();

        let select_list = inner.next().into();
        let table = inner.next().into();
        let filter = None;
        let sort = None;
        let limit = None;

        loop {
            let pair = inner.next();
            match pair {
                Some(pair) => match pair.as_rule() {
                    Rule::where_clause => filter = pair.into(),
                    Rule::order_by_clause => sort = Some(pair.into()),
                    Rule::limit_clause => limit = Some(pair.into()),
                },
                None => break,
            }
        }

        SelectQuery {
            select_list,
            table,
            filter,
            sort,
            limit,
        }
    }
}
