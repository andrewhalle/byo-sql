use super::{Datatype, Identifier, Listable};

/// TODO quick description.
///
/// TODO long description.
#[derive(Debug)]
pub struct CreateTableColumn<'input> {
    name: Identifier<'input>,
    datatype: Datatype,
}

use crate::parse::Rule;
use pest::iterators::Pair;

impl<'input> From<Pair<'input, Rule>> for CreateTableColumn<'input> {
    fn from(create_table_column: Pair<'input, Rule>) -> Self {
        assert_eq!(create_table_column.as_rule(), Rule::create_table_column);

        let mut inner = create_table_column.into_inner();
        let name = inner.next().unwrap().into();
        let datatype = inner.next().unwrap().into();

        CreateTableColumn { name, datatype }
    }
}

impl<'input> Listable for CreateTableColumn<'input> {
    fn get_rule() -> Rule {
        Rule::create_table_column_list
    }
}
