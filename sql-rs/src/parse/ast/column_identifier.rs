use super::{Column, Identifier};

/// Uniquely identifies a column.
///
/// Either the column name alone is sufficient (if the column name is globally unique) or the
/// column is specified with `alias.column`.
#[derive(Debug)]
pub struct ColumnIdentifier<'input> {
    pub alias: Option<Identifier<'input>>,
    pub name: Column<'input>,
}

impl<'input> ColumnIdentifier<'input> {
    pub fn as_string(&self) -> String {
        match &self.alias {
            None => self.name.as_string(),
            Some(a) => format!("{}.{}", a.0, self.name.as_string()),
        }
    }
}

use crate::parse::Rule;
use pest::iterators::Pair;

impl<'input> From<Pair<'input, Rule>> for ColumnIdentifier<'input> {
    fn from(column_identifier: Pair<'input, Rule>) -> Self {
        assert_eq!(column_identifier.as_rule(), Rule::column_identifier);

        let mut inner = column_identifier.into_inner();
        let first = inner.next().unwrap();
        match first.as_rule() {
            Rule::identifier => {
                let alias = first.into();
                let name = inner.next().unwrap().into();

                ColumnIdentifier {
                    name,
                    alias: Some(alias),
                }
            }
            _ => {
                let name = first.into();

                ColumnIdentifier { name, alias: None }
            }
        }
    }
}
