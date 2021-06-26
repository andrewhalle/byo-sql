/// An identifier.
///
/// Used for either a table name, a column name, or an alias.
#[derive(Debug)]
pub struct Identifier<'input>(pub &'input str);

use crate::parse::Rule;
use pest::iterators::Pair;

impl<'input> From<Pair<'input, Rule>> for Identifier<'input> {
    fn from(identifier: Pair<'input, Rule>) -> Self {
        assert_eq!(identifier.as_rule(), Rule::identifier);

        Identifier(identifier.as_str())
    }
}
