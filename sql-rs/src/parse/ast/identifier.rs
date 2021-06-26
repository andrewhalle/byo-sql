/// An identifier.
///
/// Used for either a table name, a column name, or an alias.
#[derive(Debug)]
pub struct Identifier<'input>(pub &'input str);

use pest::iterators::Pair;

use crate::parse::Rule;

impl<'input> From<Pair<'input, Rule>> for Identifier<'input> {
    fn from(pair: Pair<'input, Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::identifier);

        Identifier(pair.as_str())
    }
}
