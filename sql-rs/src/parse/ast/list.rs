use crate::parse::Rule;

/// Helper trait for parsing a repeated rule into a Vec via List.
// seal trait?
pub trait Listable {
    fn get_rule() -> Rule;
}

/// TODO quick description.
///
/// TODO long description.
#[derive(Debug)]
pub struct List<T>(pub Vec<T>);

use pest::iterators::Pair;

impl<'input, T> From<Pair<'input, Rule>> for List<T>
where
    T: From<Pair<'input, Rule>> + Listable,
{
    fn from(list: Pair<'input, Rule>) -> Self {
        assert_eq!(list.as_rule(), T::get_rule());

        List(list.into_inner().map(From::from).collect())
    }
}
