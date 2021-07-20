use std::fmt::{Display, Formatter};

use super::evaluate;
use crate::data::{Database, Table};
use crate::parse::ast::SelectQuery;

pub type Success = Table;

#[derive(Debug)]
pub struct Error;

impl Display for Error {
    fn fmt(&self, _f: &mut Formatter) -> Result<(), std::fmt::Error> {
        todo!()
    }
}

type QueryResult = Result<Success, Error>;

impl Database {
    pub fn execute_select(&self, query: SelectQuery<'_>) -> QueryResult {
        // start with the root table
        let mut result = self.find_table(query.table.root_table.name.0).clone();
        result.prefix_column_names(&format!("{}.", query.table.root_table.as_str()));

        /* XXX
        for join in query.table.joins {
            let table = self.find_table(join.table.name.0);
            let mut table = SelectQueryResult {
                columns: table
                    .columns
                    .iter()
                    .map(|c| SelectQueryResultColumn::from(join.table.name.0.to_owned(), c))
                    .collect(),
                rows: table.rows.clone(),
                table_alias_map: new_alias_map(&join.table),
            };
            result.join(&mut table, &join);
        }
        */

        if let Some(filter) = &query.filter {
            result.filter(|evaluation_context| {
                evaluate(filter, Some(evaluation_context), None).is_true()
            });
        }

        /*
        if let Some(sort) = &query.sort {
            result.sort(sort);
        }

        if let Some(rows) = &query.limit {
            // XXX evaluate, need to re-think this
            result.limit(evaluate(rows).as_number() as usize);
        }

        let result = result.select(query.select_list);
        */

        Ok(result)
    }
}
