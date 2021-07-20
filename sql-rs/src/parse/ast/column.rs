use super::Identifier;

/// Identifies a column (or set of columns) once a table is specified.
#[derive(Debug)]
pub enum Column<'input> {
    Star,
    Ident(Identifier<'input>),
}

impl<'input> Column<'input> {
    pub fn as_string(&self) -> String {
        match self {
            Column::Star => "*".to_owned(),
            Column::Ident(i) => i.0.to_owned(),
        }
    }
}

use crate::parse::Rule;
use pest::iterators::Pair;

impl<'input> From<Pair<'input, Rule>> for Column<'input> {
    fn from(column: Pair<'input, Rule>) -> Self {
        assert_eq!(column.as_rule(), Rule::column);

        match column.as_str() {
            "*" => Column::Star,
            _ => {
                let mut inner = column.into_inner();

                Column::Ident(inner.next().unwrap().into())
            }
        }
    }
}
