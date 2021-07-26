use std::cmp::Reverse;
use std::fmt::{Display, Formatter};

use super::{evaluate, evaluate_column};
use crate::data::{Database, Table};
use crate::parse::ast::{OrderByDirection, SelectQuery};

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

        for join in &query.table.joins {
            let mut table = self.find_table(join.table.name.0).clone();
            table.prefix_column_names(&format!("{}.", join.table.as_str()));

            result.join(
                table,
                |evaluation_context| {
                    evaluate(&join.condition, Some(evaluation_context), None).is_true()
                },
                join.kind,
            );
        }

        if let Some(filter) = &query.filter {
            result.filter(|evaluation_context| {
                evaluate(filter, Some(evaluation_context), None).is_true()
            });
        }

        if let Some(sort) = &query.sort {
            match sort.direction {
                OrderByDirection::Asc => result.sort(|evaluation_context| {
                    evaluate(&sort.expr, Some(evaluation_context), None)
                }),
                OrderByDirection::Desc => result.sort(|evaluation_context| {
                    Reverse(evaluate(&sort.expr, Some(evaluation_context), None))
                }),
            }
        }

        if let Some(rows) = &query.limit {
            result.limit(evaluate(rows, None, None).as_number() as usize);
        }

        let result = result.select(
            |columns| {
                let mut new_columns = Vec::new();

                for expr in &query.select_list {
                    new_columns.push(evaluate_column(expr, columns));
                }

                new_columns
            },
            |columns, rows| rows.clone(),
        );

        Ok(result)
    }
}
