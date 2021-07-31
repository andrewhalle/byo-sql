use super::{Identifier, Listable, Literal};

/// TODO short description.
///
/// TODO long description.
#[derive(Debug)]
pub struct Assignment<'input> {
    pub column: Identifier<'input>,
    pub value: Literal<'input>,
}

use crate::parse::Rule;
use pest::iterators::Pair;

impl<'input> From<Pair<'input, Rule>> for Assignment<'input> {
    fn from(assignment: Pair<'input, Rule>) -> Self {
        assert_eq!(assignment.as_rule(), Rule::assignment);

        let mut inner = assignment.into_inner();

        let column = inner.next().unwrap().into();
        let value = inner.next().unwrap().into();

        Assignment { column, value }
    }
}

impl<'input> Listable for Assignment<'input> {
    fn get_rule() -> Rule {
        Rule::assignment_list
    }
}
