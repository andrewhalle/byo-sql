use super::Identifier;

/// A table name with a possible alias.
///
/// Used when defining which tables a select query operates on
/// and establishes the context for expression evaluation using alias.column syntax.
#[derive(Debug)]
pub struct TableIdentifier<'input> {
    name: Identifier<'input>,
    alias: Option<Identifier<'input>>,
}

use crate::parse::Rule;
use pest::iterators::Pair;

impl<'input> From<Pair<'input, Rule>> for TableIdentifier<'input> {
    fn from(table_identifier: Pair<'input, Rule>) -> Self {
        assert_eq!(table_identifier.as_rule(), Rule::table_identifier);

        let mut inner = table_identifier.into_inner();
        let name = inner.next().unwrap().into();
        let alias = inner.next().map(Identifier::from);

        TableIdentifier { name, alias }
    }
}
