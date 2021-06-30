use super::{CreateTableColumn, Identifier, List};

/// TODO quick description.
///
/// TODO long description.
#[derive(Debug)]
pub struct CreateTableQuery<'input> {
    pub table_name: Identifier<'input>,
    pub columns: Vec<CreateTableColumn<'input>>,
}

use crate::parse::Rule;
use pest::iterators::Pair;

impl<'input> From<Pair<'input, Rule>> for CreateTableQuery<'input> {
    fn from(create_table_query: Pair<'input, Rule>) -> Self {
        assert_eq!(create_table_query.as_rule(), Rule::create_table_query);

        let mut inner = create_table_query.into_inner();
        let table_name = inner.next().unwrap().into();
        let columns: List<CreateTableColumn<'input>> = inner.next().unwrap().into();

        CreateTableQuery {
            table_name,
            columns: columns.0,
        }
    }
}
