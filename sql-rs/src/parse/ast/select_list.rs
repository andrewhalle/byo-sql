use super::Expression;

/// TODO quick description.
///
/// TODO long description.
#[derive(Debug)]
pub struct SelectList<'input>(Vec<Expression<'input>>);

use crate::parse::Rule;
use pest::iterators::Pair;

impl<'input> From<Pair<'input, Rule>> for SelectList<'input> {
    fn from(select_list: Pair<'input, Rule>) -> Self {
        assert_eq!(select_list.as_rule(), Rule::select_list);

        select_list.into_inner().map(From::from).collect()
    }
}
