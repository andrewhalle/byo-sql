use super::Expression;

/// TODO quick description.
///
/// TODO long description.
#[derive(Debug)]
pub struct OrderBy<'input> {
    pub expr: Expression<'input>,
    pub direction: OrderByDirection,
}

#[derive(Debug)]
pub enum OrderByDirection {
    Asc,
    Desc,
}

impl Default for OrderByDirection {
    fn default() -> Self {
        OrderByDirection::Asc
    }
}

use crate::parse::Rule;
use pest::iterators::Pair;

impl<'input> From<Pair<'input, Rule>> for OrderBy<'input> {
    fn from(order_by: Pair<'input, Rule>) -> Self {
        assert_eq!(order_by.as_rule(), Rule::order_by_clause);

        let mut inner = order_by.into_inner();
        let expr = inner.next().unwrap().into();
        let direction = inner.next().map(From::from).unwrap_or_default();

        OrderBy { expr, direction }
    }
}

impl<'input> From<Pair<'input, Rule>> for OrderByDirection {
    fn from(direction: Pair<'input, Rule>) -> Self {
        assert_eq!(direction.as_rule(), Rule::direction);

        match direction.as_str() {
            "asc" => OrderByDirection::Asc,
            "desc" => OrderByDirection::Desc,
            _ => unreachable!(),
        }
    }
}
