use super::{Expression, TableIdentifier};

/// TODO quick description.
///
/// TODO long description.
#[derive(Debug)]
pub struct Join<'input> {
    pub kind: JoinKind,
    pub table: TableIdentifier<'input>,
    pub condition: Expression<'input>,
}

#[derive(Debug)]
pub enum JoinKind {
    Inner,
    Left,
    Right,
}

use crate::parse::Rule;
use pest::iterators::Pair;

impl<'input> From<Pair<'input, Rule>> for Join<'input> {
    fn from(join: Pair<'input, Rule>) -> Self {
        assert_eq!(join.as_rule(), Rule::join_clause);

        let mut inner = join.into_inner();
        let kind = inner.next().unwrap().into();
        let table = inner.next().unwrap().into();
        let condition = inner.next().unwrap().into();

        Join {
            kind,
            table,
            condition,
        }
    }
}

impl<'input> From<Pair<'input, Rule>> for JoinKind {
    fn from(join_type: Pair<'input, Rule>) -> Self {
        assert_eq!(join_type.as_rule(), Rule::join_type);

        match join_type.into_inner().next().unwrap().as_rule() {
            Rule::inner_join => JoinKind::Inner,
            Rule::left_join => JoinKind::Left,
            Rule::right_join => JoinKind::Right,
            _ => unreachable!(),
        }
    }
}
